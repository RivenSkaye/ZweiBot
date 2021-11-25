use chrono::Utc;
use serenity::{
    async_trait,
    client::{bridge::gateway::GatewayIntents, Client},
    framework,
    http::Http,
    model::{channel::Message, event::ResumedEvent, gateway::Ready, id::UserId},
    prelude::*,
};
use std::collections::HashSet;

mod zwei_conf;

#[macro_use]
extern crate lazy_static;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} connected to Discord at {}", ready.user.name, Utc::now())
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        println!("Reconnected at {}", Utc::now())
    }

    async fn message(&self, _: Context, msg: Message) {
        println!("Received a message!\n{:?}", msg.content);
    }
}

#[framework::standard::macros::hook]
async fn prefix(_ctx: &Context, _msg: &Message) -> Option<String> {
    // Function for dynamic prefixing, currently unused.
    // rip out its guts and Frankenstein it when the time comes!
    Some(String::from(";"))
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
    let fw = framework::standard::StandardFramework::new().configure(|c| {
        c.prefix(";")
            .on_mention(Some(self_id))
            .dynamic_prefix(prefix)
            .with_whitespace(true)
            .owners(owners)
            .case_insensitivity(true)
    });
    let mut bot = Client::builder(&conf.token)
        .event_handler(Handler)
        .framework(fw)
        .intents(GatewayIntents::all())
        .await
        .expect("Zwei is feeling special today");
    bot.start().await.expect("Bot no start");
    // do code and get rekt
}
