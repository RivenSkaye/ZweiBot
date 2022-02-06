use chrono::Utc;
use serenity::{
    async_trait,
    client::{
        bridge::gateway::{GatewayIntents, ShardManager},
        Client,
    },
    framework,
    framework::standard::{macros::help, Args, CommandGroup, CommandResult, HelpOptions},
    http::Http,
    model::{channel::Message, event::ResumedEvent, gateway::Ready, id::UserId},
    prelude::*,
    Result as SerenityResult,
};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

mod commands;
mod zwei_conf;

#[macro_use]
extern crate lazy_static;

pub struct ShardManagerContainer;
impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

pub struct ZweiData;
impl TypeMapKey for ZweiData {
    type Value = HashMap<String, i64>;
}

pub struct ZweiOwners;
impl TypeMapKey for ZweiOwners {
    type Value = HashSet<UserId>;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        let stime = Utc::now();
        let time = stime.timestamp();
        {
            let mut data = ctx.data.write().await;
            let lt = data
                .get_mut::<ZweiData>()
                .expect("Couldn't get lifetime info...")
                .entry(String::from("Init"))
                .or_insert(time);
            *lt = time;
        }
        println!("{} connected to Discord at {}", ready.user.name, stime)
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        println!("Reconnected at {}", Utc::now())
    }
}

#[framework::standard::macros::hook]
async fn prefix(_ctx: &Context, _msg: &Message) -> Option<String> {
    // Function for dynamic prefixing, currently unused.
    // rip out its guts and Frankenstein it when the time comes!
    Some(String::from(";"))
}

pub fn sanitize_txt(txt: &str) -> String {
    // Thanks a lot to Acrimon for making this not shit
    let mut sanitized = String::with_capacity(txt.len() * 4 / 3);
    txt.chars().for_each(|c| match c {
        '\\' => sanitized.push_str("\\\\"),
        '~' => sanitized.push_str("\\~"),
        '_' => sanitized.push_str("\\_"),
        '*' => sanitized.push_str("\\*"),
        '|' => sanitized.push_str("\\|"),
        '`' => sanitized.push_str("\\`"),
        '<' => sanitized.push_str("\\<"),
        '>' => sanitized.push_str("\\>"),
        '[' => sanitized.push_str("\\["),
        ']' => sanitized.push_str("\\]"),
        _ => sanitized.push(c),
    });
    sanitized
}

pub async fn get_name(msg: &Message, ctx: &Context, mem: UserId) -> SerenityResult<String> {
    let guild = msg.guild(ctx).await;
    if let Some(g) = guild {
        let gmem = g.member(ctx, mem).await?;
        return Ok(gmem.display_name().into_owned());
    } else {
        let user = mem.to_user(ctx).await?;
        return Ok(format!("{:}#{:}", user.name, user.discriminator));
    }
}

pub async fn try_dm(
    ctx: &Context,
    user: UserId,
    msg: impl std::fmt::Display,
) -> SerenityResult<()> {
    let chan = user.create_dm_channel(ctx).await?;
    chan.say(ctx, msg).await?;
    Ok(())
}

pub async fn get_guildname(msg: &Message, ctx: &Context) -> String {
    msg.guild_id
        .unwrap()
        .name(ctx)
        .await
        .unwrap_or(String::from(""))
}

#[help]
async fn zwei_help(
    ctx: &Context,
    msg: &Message,
    args: Args,
    opts: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    framework::standard::help_commands::with_embeds(ctx, msg, args, opts, groups, owners).await;
    Ok(())
}

#[tokio::main]
async fn main() {
    let conf = &zwei_conf::CONF;
    let http = Http::new_with_token(&conf.token);
    let owners = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::<UserId>::new();
            if let Some(team) = info.team {
                owners.extend(team.members.iter().map(|m| m.user.id));
            } else {
                owners.insert(info.owner.id);
            }
            owners.extend(conf.owners.iter().map(|o| UserId(*o)));
            owners
        }
        Err(why) => panic!(
            "Couldn't find owners for the supplied token!\nToken: {}\n{:?}",
            &conf.token, why
        ),
    };
    let self_id = http.get_current_user().await.unwrap().id;
    let fw = framework::standard::StandardFramework::new()
        .configure(|c| {
            c.prefix("]")
                .on_mention(Some(self_id))
                .dynamic_prefix(prefix)
                .with_whitespace(true)
                .owners(owners.clone())
                .case_insensitivity(true)
        })
        .help(&ZWEI_HELP)
        .group(&commands::modtools::MODTOOLS_GROUP)
        .group(&commands::management::MANAGEMENT_GROUP);
    let mut bot = Client::builder(&conf.token)
        .event_handler(Handler)
        .framework(fw)
        .intents(GatewayIntents::all())
        .await
        .expect("Zwei is feeling special today");
    {
        let mut data = bot.data.write().await;
        data.insert::<ShardManagerContainer>(bot.shard_manager.clone());
        let mut zd = HashMap::new();
        zd.insert("Init".to_string(), Utc::now().timestamp());
        zd.insert("id".to_string(), i64::from(self_id));
        data.insert::<ZweiData>(zd);
        data.insert::<ZweiOwners>(owners.clone());
    }
    let shard_manager = bot.shard_manager.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("I can't listen for SIGKILL, HALP!");
        shard_manager.lock().await.shutdown_all().await;
    });

    bot.start()
        .await
        .expect("Zwei is stuck in the Fringe Dimension. Try again...");
    // do code and get rekt
}
