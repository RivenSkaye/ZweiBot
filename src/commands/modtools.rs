use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use tokio::time::{sleep, Duration};

#[command]
#[owners_only]
#[max_args(1)]
async fn exit(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    args.trimmed();
    let time: u64 = args.parse::<u64>()?;
    msg.reply(ctx, format!("I'm taking a dirt nap in {:} seconds.", time))
        .await?;
    sleep(Duration::from_secs(time)).await;
    std::process::exit(0)
}

#[group("modtools")]
#[commands(exit)]
struct ModTools;
