use chrono::Utc;
use serenity::{
    async_trait, framework,
    http::Http,
    model::{event::ResumedEvent, gateway::Ready, id::UserId, user},
    prelude::*,
    utils, Client,
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
    let mut bot = Client::builder(&conf.token).await.expect("REEEEE");
    bot.start().await.expect("Bot no start");
    // do code and get rekt
}
