use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use tokio::time::{sleep, Duration};

use crate::ShardManagerContainer;

#[command]
#[owners_only]
#[max_args(1)]
async fn exit(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    args.trimmed();
    let time: u64 = args.parse::<u64>()?;
    let botdata = ctx.data.read().await;

    if let Some(manager) = botdata.get::<ShardManagerContainer>() {
        msg.reply(ctx, format!("I'm taking a dirt nap in {:} seconds.", time))
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

async fn purge(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    Ok(())
}

#[group("modtools")]
#[commands(exit, purge)]
struct ModTools;
