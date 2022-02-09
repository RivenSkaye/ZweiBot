use chrono::Utc;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use tokio::time::{sleep, Duration};

use crate::{get_name, ShardManagerContainer, ZweiData, ZweiOwners};

#[command]
#[owners_only]
#[max_args(1)]
#[aliases("shutdown", "panic", "die", "sleep")]
#[description = "Shuts down the bot, owner only. Optionally takes a time in seconds to wait, defaults to 1 second."]
#[example = "shutdown 5"]
#[example = "sleep"]
#[help_available]
async fn exit(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    args.trimmed();
    let time: u64 = args.parse::<u64>().unwrap_or(1);
    args.advance();
    let botdata = ctx.data.read().await;

    if let Some(manager) = botdata.get::<ShardManagerContainer>() {
        msg.reply(ctx, format!("I'm taking a nap in {:} seconds.", time))
            .await?;
        sleep(Duration::from_secs(time)).await;
        manager.lock().await.shutdown_all().await;
    } else {
        msg.reply(ctx, "I've lost control of Kuro, I'm stopping RIGHT NOW!")
            .await?;
        std::process::exit(0)
    }
    Ok(())
}

#[command]
#[description = "Prints the bot's total running time since the `on_ready` event fired."]
async fn uptime(ctx: &Context, msg: &Message) -> CommandResult {
    let botdata = ctx.data.read().await;
    if let Some(lifetime) = botdata.get::<ZweiData>() {
        let now = Utc::now().timestamp();
        let starttime = lifetime.get(&String::from("Init")).unwrap();
        let diff = now - *starttime;
        let secs = diff % 60;
        let mins = (diff % 3600) / 60;
        let hours = diff / 3600;
        let difftxt = format!(
            "I've been running around for {:} hours, {:} minutes and {:} seconds now.",
            hours, mins, secs
        );
        msg.reply(ctx, difftxt).await?;
    } else {
        msg.reply(
            ctx,
            "I've been in Lost Blue for so long that I can't even remember when I got here...",
        )
        .await?;
    }
    Ok(())
}

#[command]
#[description = "Get the seconds-exact current UTC time, disregarding leap seconds."]
#[help_available]
async fn now(ctx: &Context, msg: &Message) -> CommandResult {
    let now = Utc::now().timestamp() - Utc::today().and_hms(0, 0, 0).timestamp();
    let diff = now;
    let secs = diff % 60;
    let mins = (diff % 3600) / 60;
    let hours = diff / 3600;
    let difftxt = format!(
        "It's now {:} hours, {:} minutes and {:} seconds.",
        hours, mins, secs
    );
    msg.reply(ctx, difftxt).await?;
    Ok(())
}

#[command]
#[description = "Provides information about the amazing people behind the bot."]
#[help_available]
#[aliases("credits", "creators")]
async fn owners(ctx: &Context, msg: &Message) -> CommandResult {
    let mut owner_ids = Vec::new();
    let mut ownernames = String::from(
        "These are the wonderful people who wrote me or were guinea pigs for testing!",
    );
    let botdata = ctx.data.read().await;
    if let Some(owners) = botdata.get::<ZweiOwners>() {
        owners.iter().for_each(|o| owner_ids.push(o));
    }
    while owner_ids.len() > 0 {
        let name = get_name(msg, ctx, *owner_ids.pop().unwrap()).await?;
        ownernames.push_str("\n- ");
        ownernames.push_str(&*name);
    }
    msg.reply(ctx, ownernames).await?;
    Ok(())
}

#[group("Misc")]
#[commands(exit, uptime, now, owners)]
#[summary = "Miscellaneous commands for bot management and statistics."]
struct Misc;