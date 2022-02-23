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
    prelude::*, // also implies tokio Mutex
    utils::Colour,
    Result as SerenityResult,
};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

mod commands;
mod db;
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

pub struct ZweiPrefixes;
impl TypeMapKey for ZweiPrefixes {
    type Value = HashMap<u64, String>;
}

pub struct ZweiOwners;
impl TypeMapKey for ZweiOwners {
    type Value = HashSet<UserId>;
}

use db::ZweiDBConn;

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

/// Function to resolve an ID into a username. Attempts to get the proper
/// _display name_ for the context first, but falls back to the normal
/// username if a nickname isn't set or the context is not a guild.
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

/// Function to resolve the textual name for the guild from the message.
pub async fn get_guildname(msg: &Message, ctx: &Context) -> String {
    msg.guild_id
        .unwrap()
        .name(ctx)
        .await
        .unwrap_or(String::from(""))
}

/// Function to get the prefix for the current context.
/// Returns either the default `;` in DMs or when the guild has no prefix,
/// or returns the guild-specific prefix if one is set.
pub async fn get_prefix(msg: &Message, ctx: &Context) -> String {
    let def = String::from(";");
    let botdata = ctx.data.read().await;
    botdata
        .get::<ZweiPrefixes>()
        .and_then(|d| msg.guild_id.and_then(|g| d.get(&g.0)))
        .cloned()
        .unwrap_or(def)
}

/// Function to try sending a direct message to a user. This is used to try
/// and notify a user that they've been kicked or banned, but can also be
/// used for otherwise contacting a user directly.
///
/// # Arguments
///
/// * `ctx` - The command context for the direct message.
/// * `user` - The `UserId` for the user to DM.
/// * `title` - Anything that implements [`std::fmt::Display`].
/// * `msg` - The message body to send, must implement [`std::fmt::Display`].
///
/// Examples can be found in [`commands::modtools`] in `kick` and `ban`.
pub async fn try_dm(
    ctx: &Context,
    user: UserId,
    title: impl std::fmt::Display,
    msg: impl std::fmt::Display,
) -> SerenityResult<()> {
    let color =
        Colour::from(u32::from_str_radix(&zwei_conf::CONF.ok_color.replace("#", ""), 16).unwrap());
    let chan = user.create_dm_channel(ctx).await?;
    chan.send_message(ctx, |mes| {
        mes.embed(|e| e.color(color).title(title).description(msg))
    })
    .await?;
    Ok(())
}

/// Function to sanitize text and escape markdown. Especially useful when
/// dealing with clowns that think it's funny to make a bot post broken
/// messages rather than correctly displaying their name.
/// Can be used to sanitize any arbitrary string slice.
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

/// Central function to send an embed indicating something went wrong.
/// This is the version that's not too descriptive.
///
/// # Arguments
///
/// * `ctx` - Command context
/// * `msg` - The message that invoked the command leading to the error.
/// * `errtxt` - Anything implementing `std::fmt::Display` as textual
///              indication of what went wrong.
pub async fn send_err(
    ctx: &Context,
    msg: &Message,
    errtxt: impl std::fmt::Display,
) -> CommandResult {
    Ok(send_err_titled(ctx, msg, "Something went wrong!", errtxt).await?)
}

/// Central function to send an embed indicating something went wrong.
/// Allows for setting a title as well, which could be used to provide more
/// information about what went wrong or how a command was used incorrectly.
///
/// # Arguments
///
/// * `ctx` - Command context
/// * `msg` - The message that invoked the command leading to the error.
/// * `title` - Anything that implements [`std::fmt::Display`], like the
///             command name or some other descriptive heading.
/// * `errtxt` - Anything implementing [`std::fmt::Display`] as textual
///              indication of what went wrong.
pub async fn send_err_titled(
    ctx: &Context,
    msg: &Message,
    title: impl std::fmt::Display,
    errtxt: impl std::fmt::Display,
) -> CommandResult {
    let color =
        Colour::from(u32::from_str_radix(&zwei_conf::CONF.err_color.replace("#", ""), 16).unwrap());
    msg.channel_id
        .send_message(ctx, |mes| {
            mes.embed(|e| e.color(color).title(title).description(errtxt))
        })
        .await?;
    Ok(())
}

/// Function to indicate successful processing of a command. Uses Zwei's
/// pretty colors to indicate everything is good.
///
/// # Arguments
///
/// * `ctx` - Command context
/// * `msg` - The message that invoked the command that was completed.
/// * `title` - Anything that implements [`std::fmt::Display`], like the
///             command name or some other descriptive heading.
/// * `msgtxt` - Anything implementing [`std::fmt::Display`] as textual message
///              returning information about command execution to the user.
pub async fn send_ok(
    ctx: &Context,
    msg: &Message,
    title: impl std::fmt::Display,
    msgtxt: impl std::fmt::Display,
) -> CommandResult {
    let color =
        Colour::from(u32::from_str_radix(&zwei_conf::CONF.ok_color.replace("#", ""), 16).unwrap());
    msg.channel_id
        .send_message(ctx, |mes| {
            mes.embed(|e| e.color(color).title(title).description(msgtxt))
        })
        .await?;
    Ok(())
}

#[help]
#[lacking_permissions = "hide"]
#[lacking_ownership = "hide"]
#[embed_error_colour("#9A48C9")]
#[embed_success_colour("#B82748")]
#[strikethrough_commands_tip_in_guild = ""]
#[strikethrough_commands_tip_in_dm = ""]
#[group_prefix = "All commands include"]
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

/// Function for dynamic prefixing. Checks the cache for custom prefixes,
/// if none is found for this server, returns the default instead.
#[framework::standard::macros::hook]
async fn prefix(ctx: &Context, msg: &Message) -> Option<String> {
    Some(get_prefix(msg, ctx).await)
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
            c.prefix("")
                .on_mention(Some(self_id))
                .dynamic_prefix(prefix)
                .with_whitespace(true)
                .owners(owners.clone())
                .case_insensitivity(true)
        })
        .help(&ZWEI_HELP)
        .group(&commands::modtools::MODTOOLS_GROUP)
        .group(&commands::misc::MISC_GROUP)
        .group(&commands::misc::PREFIX_GROUP);
    let mut bot = Client::builder(&conf.token)
        .event_handler(Handler)
        .framework(fw)
        .intents(GatewayIntents::all())
        .await
        .expect("Zwei is feeling special today");
    let arcsqlite = Arc::new(Mutex::new(
        rusqlite::Connection::open(zwei_conf::DATADIR.join(&conf.database))
            .expect("Can't open the database!"),
    ));
    {
        let mut data = bot.data.write().await;
        data.insert::<ShardManagerContainer>(bot.shard_manager.clone());
        let mut zd = HashMap::new();
        zd.insert("Init".to_string(), Utc::now().timestamp());
        zd.insert("id".to_string(), i64::from(self_id));
        data.insert::<ZweiData>(zd);
        data.insert::<ZweiOwners>(owners.clone());
        data.insert::<ZweiDBConn>(arcsqlite);
        data.insert::<ZweiPrefixes>(
            db::get_all_prefixes(
                &rusqlite::Connection::open(zwei_conf::DATADIR.join(&conf.database))
                    .expect("Can't open the database!"),
            )
            .unwrap(),
        )
    }
    let shard_manager = bot.shard_manager.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("I can't listen for SIGKILL, HALP!");
        shard_manager.lock().await.shutdown_all().await;
    });

    if let Err(death) = bot.start().await {
        println!("Zwei did not exit cleanly!\n{:}", death);
        bot.shard_manager.lock().await.shutdown_all().await;
    }
    // Perform exit logic here. Save config changes, mangle pending DB ops, etc
}
