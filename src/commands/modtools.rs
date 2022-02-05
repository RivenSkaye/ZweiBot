use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::{
    id::{MessageId, UserId},
    prelude::*,
};
use serenity::prelude::*;

use crate::get_name;

#[command]
#[required_permissions("MANAGE_MESSAGES")]
#[only_in("guilds")]
#[min_args(1)]
#[max_args(2)]
#[aliases("prune")]
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

#[command]
#[required_permissions("KICK_MEMBERS")]
#[max_args(1)]
async fn kick(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    args.trimmed();
    let mem_id = args.parse::<UserId>().unwrap_or_default();

    if mem_id == UserId::default() {
        msg.reply(ctx, "Please specify a user to kick by mention or ID.")
            .await?;
        return Ok(());
    }

    let fullname = get_name(msg, ctx).await?;
    if let Err(_) = msg.guild_id.unwrap_or_default().kick(ctx, mem_id).await {
        msg.reply(
            ctx,
            format!(
                "I can't kick {:}, please check if their roles are higher than mine.",
                fullname
            ),
        )
        .await?;
    } else {
        msg.reply(
            ctx,
            format!("I sent {:} away. Be careful if they return.", fullname),
        )
        .await?;
    }
    Ok(())
}

#[command]
#[required_permissions("BAN_MEMBERS")]
#[max_args(2)]
async fn ban(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    args.trimmed();
    let mem_id = args.parse::<UserId>().unwrap_or_default();

    if mem_id == UserId::default() {
        msg.reply(ctx, "Please specify a user to ban by mention or ID.")
            .await?;
        return Ok(());
    }

    let fullname = get_name(msg, ctx).await?;
    let days = args.parse::<u8>().unwrap_or(0);
    if let Err(_) = msg
        .guild_id
        .unwrap_or_default()
        .ban(ctx, mem_id, days)
        .await
    {
        msg.reply(
            ctx,
            format!(
                "I can't ban {:}, please check if their roles are higher than mine.",
                fullname
            ),
        )
        .await?;
    } else {
        msg.reply(
            ctx,
            format!(
                "I sent {:} to Lost Blue. You won't see them again.",
                fullname
            ),
        )
        .await?;
    }
    Ok(())
}

#[group("modtools")]
#[commands(purge, kick, ban)]
#[summary = "Commands for moderators and admins of a server."]
#[only_in("guilds")]
struct ModTools;
