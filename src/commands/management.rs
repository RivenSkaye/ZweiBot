use chrono::Utc;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::{
    id::{MessageId, UserId},
    prelude::*,
};
use serenity::prelude::*;
use tokio::time::{sleep, Duration};

use crate::{get_name, ShardManagerContainer, ZweiLifeTimes};

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
    if let Some(lifetime) = botdata.get::<ZweiLifeTimes>() {
        let now = Utc::now();
        let timetotal = lifetime.get(&String::from("Init")).unwrap();
        let diff = now - *timetotal;
        let difftxt = match diff.to_std() {
            Err(_) => String::from(
                "Sorry! I've forgotten to keep track of how long I've been climbing the tower.",
            ),
            Ok(_) => format!(
                "I've been running around for {:} hours, {:} minutes and {:} seconds now.",
                diff.num_hours(),
                diff.num_minutes() % 60,
                diff.num_seconds() % 60
            ),
        };
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

#[group("management")]
#[commands(exit, uptime)]
#[summary = "Miscellaneous commands for bot management and stats."]
struct Management;
