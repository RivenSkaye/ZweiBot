use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::{id::MessageId, prelude::*};
use serenity::prelude::*;
use tokio::time::{sleep, Duration};

use crate::ShardManagerContainer;

#[command]
#[owners_only]
#[max_args(1)]
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
#[required_permissions("MANAGE_MESSAGES")]
#[only_in("guilds")]
#[min_args(1)]
#[max_args(2)]
async fn purge(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    args.trimmed();
    let amount: u64 = args.parse::<u64>().unwrap_or(0);

    if amount < 1 {
        msg.reply(ctx, "Could you stop trying to purge thin air?")
            .await?;
        return Ok(());
    } else if amount > 100 {
        msg.reply(ctx,
            "Please keep the amount of messages to purge somewhat manageable. Due to technical limitations, the maximum amount is 100.")
            .await?;
        return Ok(());
    }
    let to_delete = msg
        .channel_id
        .messages(&ctx.http, |m| m.before(msg.id).limit(amount))
        .await?
        .into_iter()
        .filter(|m| !m.pinned)
        .map(|m| m.id)
        .collect::<Vec<MessageId>>();

    let pinned = amount - to_delete.len() as u64;
    if pinned == amount {
        if amount > 1 {
            msg.reply(ctx, "All those messages are pinned, I can't delete them.")
                .await?;
        } else {
            msg.reply(ctx, "That message is pinned, I can't delete it.")
                .await?;
        }
        return Ok(());
    }

    let reply = match pinned {
        0 => match amount {
            1 => "Deleting the last message. _You could've done that faster manually._".to_string(),
            _ => format!("Purging the last {:} messages.", amount),
        },
        _ => format!(
            "Purging {:} out of the last {:} messages.\nThe other {:} {:} pinned.",
            amount - pinned,
            amount,
            pinned,
            if pinned == 1 { "was" } else { "were" }
        ),
    };
    msg.reply(ctx, reply).await?;

    msg.channel_id.delete_messages(&ctx.http, to_delete).await?;
    Ok(())
}

#[group("modtools")]
#[commands(exit, purge)]
struct ModTools;
