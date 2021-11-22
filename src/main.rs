use chrono::Utc;
use serenity::{
    async_trait, builder, cache, client, constants, framework,
    model::{event::ResumedEvent, gateway::Ready, prelude, user},
    prelude::*,
    utils, Client,
};

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
    let mut bot = Client::builder(&conf.token).await.expect("REEEEE");
    bot.start().await.expect("Bot no start");
    // do code and get rekt
}
