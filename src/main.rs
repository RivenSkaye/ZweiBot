use chrono::{DateTime, Utc};
use serenity::{
    async_trait,
    client::{
        bridge::gateway::{GatewayIntents, ShardManager},
        Client,
    },
    framework,
    http::Http,
    model::{channel::Message, event::ResumedEvent, gateway::Ready, id::UserId},
    prelude::*,
};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

mod commands;
mod zwei_conf;

#[macro_use]
extern crate lazy_static;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        let time = Utc::now();
        {
            let mut data = ctx.data.write().await;
            let lt = data
                .get_mut::<ZweiLifeTimes>()
                .expect("Couldn't get lifetime info...")
                .entry(String::from("Init"))
                .or_insert(time);
            *lt = time;
        }
        println!("{} connected to Discord at {}", ready.user.name, time)
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

pub struct ShardManagerContainer;
impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

pub struct ZweiLifeTimes;
impl TypeMapKey for ZweiLifeTimes {
    type Value = HashMap<String, DateTime<Utc>>;
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
                .owners(owners)
                .case_insensitivity(true)
        })
        .group(&commands::modtools::MODTOOLS_GROUP);
    let mut bot = Client::builder(&conf.token)
        .event_handler(Handler)
        .framework(fw)
        .intents(GatewayIntents::all())
        .await
        .expect("Zwei is feeling special today");
    {
        let mut data = bot.data.write().await;
        data.insert::<ShardManagerContainer>(bot.shard_manager.clone());
        let mut blank_date = HashMap::new();
        blank_date.insert("Init".to_string(), Utc::now());
        data.insert::<ZweiLifeTimes>(blank_date);
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
