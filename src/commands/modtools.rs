use serenity::{
    framework::standard::{
        macros::{command, group},
        Args, CommandResult,
    },
    model::{id::UserId, prelude::*},
    prelude::*,
    Error,
};

use crate::{get_guildname, get_name, send_err, send_err_titled, send_ok, try_dm, ZweiData};

#[command]
#[required_permissions("MANAGE_MESSAGES")]
#[only_in("guilds")]
#[num_args(1)]
#[aliases("prune", "massdelete", "massdel")]
#[description = "Deletes the specified amount of unpinned messages in the chat. Max 100."]
async fn purge(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    args.trimmed();
    let amount: u64 = args.parse::<u64>().unwrap_or(0);

    if amount < 1 {
        return send_err_titled(
            ctx,
            msg,
            "No messages!",
            "Could you stop trying to purge thin air?",
        )
        .await;
    } else if amount > 100 {
        return send_err_titled(
            ctx,
            msg,
            "Too many messages!",
            "Please keep the amount of messages to purge somewhat manageable. Due to technical limitations, the maximum amount is 100.",
        )
        .await;
    }

    let recent_messages = msg
        .channel_id
        .messages(&ctx.http, |m| m.before(msg.id).limit(amount))
        .await?;

    let mut num_pinned = 0;
    let mut to_delete = Vec::with_capacity(recent_messages.len());
    for msg in recent_messages {
        if msg.pinned {
            num_pinned += 1;
        } else {
            to_delete.push(msg.id);
        }
    }

    let reply = match (to_delete.len(), num_pinned) {
        (0, 1) => Err("That message is pinned. I can't delete it."),
        (0, _) => Err("Those messages are all pinned. I can't delete them."),
        (1, 0) => Ok("the last message. _You could've done that faster manually._".to_owned()),
        (n, 0) => Ok(format!("the last {n} messages.")),
        (n, 1) => Ok(format!(
            "{n} out of the last {amount} messages. The other one was pinned."
        )),
        (n, p) => Ok(format!(
            "{n} out of the last {amount} messages. The other {p} were pinned."
        )),
    };

    match reply {
        Ok(r) => {
            send_ok(ctx, msg, "Purging", r).await?;
            msg.channel_id.delete_messages(&ctx.http, to_delete).await?;
        }
        Err(e) => send_err(ctx, msg, e).await?,
    }

    Ok(())
}

#[command]
#[required_permissions("KICK_MEMBERS")]
#[num_args(1)]
#[description = "Kicks a member from the server. Optionally with a reason."]
#[aliases("remove", "prod", "eject")]
async fn kick(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    args.trimmed();
    let mem_id = args.parse::<UserId>().unwrap_or_default();

    if mem_id.0 == 0 {
        return send_err_titled(
            ctx,
            msg,
            "No target provided!",
            "Please give me a user mention or an ID to kick.",
        )
        .await;
    } else if mem_id.0 == msg.guild(ctx).unwrap().owner_id.0 {
        return send_err_titled(
            ctx,
            msg,
            "That's not possible!",
            "I can't kick the owner off their own server.",
        )
        .await;
    }
    let memrole = msg
        .guild(ctx)
        .unwrap()
        .member(ctx, u64::try_from(mem_id.0)?)
        .await?
        .highest_role_info(ctx)
        .unwrap();
    let botdata = ctx.data.read().await;
    if let Some(data) = botdata.get::<ZweiData>() {
        let self_id = u64::try_from(*data.get("id").unwrap())?;
        let selfrole = msg
            .guild(ctx)
            .unwrap()
            .member(ctx, self_id)
            .await?
            .highest_role_info(ctx)
            .unwrap();

        if self_id == mem_id.0 {
            msg.reply_ping(ctx, "<:ZweiAngery:844167326243880960>")
                .await?;
            return Ok(());
        } else if selfrole.1 <= memrole.1 {
            return send_err(
                ctx,
                msg,
                "I can't kick someone whose roles are equal to or higher than my own!",
            )
            .await;
        }
    };

    let fullname = get_name(msg, ctx, mem_id).await?;
    args.advance();
    let reason = args.remains().unwrap_or("You know what you did!");

    let guildname = get_guildname(msg, ctx).await;
    let _ = try_dm(
        ctx,
        mem_id,
        "<:ZweiShy:844167336336031745> Sorry!",
        format!("You were kicked from {guildname}.\nReason: {reason}"),
    )
    .await;

    if let Err(e) = msg
        .guild_id
        .unwrap_or_default()
        .kick_with_reason(ctx, mem_id, reason)
        .await
    {
        let txt = match e {
            Error::Model(ModelError::InvalidPermissions(missing_perms)) => {
                format!("please provide me with the `{missing_perms}` permission")
            }
            _ => "the provided reason was too long".to_owned(),
        };
        return send_err(ctx, msg, format!("I can't kick {fullname}, {txt}.")).await;
    }
    send_ok(
        ctx,
        msg,
        "User kicked.",
        format!("I sent {fullname} away. Be careful if they return."),
    )
    .await
}

#[command]
#[required_permissions("BAN_MEMBERS")]
#[min_args(1)]
#[aliases("yeet", "lostblue")]
async fn ban(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    args.trimmed();
    let mem_id = args.parse::<UserId>().unwrap_or_default();

    if mem_id.0 == 0 {
        return send_err_titled(
            ctx,
            msg,
            "No target provided!",
            "Please give me a user mention or an ID to ban.",
        )
        .await;
    } else if mem_id.0 == msg.guild(ctx).unwrap().owner_id.0 {
        return send_err_titled(
            ctx,
            msg,
            "That's not possible!",
            "I can't ban the owner from own server.",
        )
        .await;
    }
    let memrole = msg
        .guild(ctx)
        .unwrap()
        .member(ctx, mem_id.0)
        .await?
        .highest_role_info(ctx)
        .unwrap();
    let botdata = ctx.data.read().await;
    if let Some(data) = botdata.get::<ZweiData>() {
        let self_id = u64::try_from(*data.get("id").unwrap())?;
        let selfrole = msg
            .guild(ctx)
            .unwrap()
            .member(ctx, self_id)
            .await?
            .highest_role_info(ctx)
            .unwrap();

        if self_id == mem_id.0 {
            msg.reply_ping(ctx, "<:ZweiAngery:844167326243880960>")
                .await?;
            return Ok(());
        } else if selfrole.1 <= memrole.1 {
            return send_err(
                ctx,
                msg,
                "I can't ban someone whose roles are equal to or higher than my own!",
            )
            .await;
        }
    };

    let fullname = get_name(msg, ctx, mem_id).await?;
    args.advance();
    let days = args.parse::<u8>().unwrap_or(0);
    args.advance();
    let reason = args.remains().unwrap_or("You know what you did!");

    let guildname = get_guildname(msg, ctx).await;
    let _ = try_dm(
        ctx,
        mem_id,
        "<:ZweiShy:844167336336031745> Sorry!",
        format!("You were banned from {guildname}.\nReason: {reason}",),
    )
    .await;

    let realdays: u8 = if days > 7 {
        send_err_titled(
            ctx,
            msg,
            "I can't change history",
            "and Discord doesn't allow deleting more than 7 days when banning.\nDefaulting to that instead.",
        )
        .await?;
        7
    } else {
        days
    };
    if let Err(e) = msg
        .guild_id
        .unwrap_or_default()
        .ban_with_reason(ctx, mem_id, realdays, reason)
        .await
    {
        let txt = match e {
            Error::Model(ModelError::InvalidPermissions(missing_perms)) => {
                format!("please provide me with the `{missing_perms}` permission")
            }
            _ => "the provided reason was too long".to_owned(),
        };
        return send_err(ctx, msg, format!("I can't ban {fullname}, {txt}.")).await;
    }
    return send_ok(
        ctx,
        msg,
        "User banned.",
        format!("I sent {fullname} off to Lost Blue. You won't see them again."),
    )
    .await;
}

#[command]
#[num_args(1)]
async fn warn(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    args.trimmed();
    let mem_id = args.parse::<UserId>().unwrap_or_default();

    if mem_id.0 == 0 {
        return send_err_titled(
            ctx,
            msg,
            "No target provided!",
            "Please give me a user mention or an ID to warn.",
        )
        .await;
    } else if mem_id.0 == msg.guild(ctx).unwrap().owner_id.0 {
        return send_err_titled(
            ctx,
            msg,
            "That's not useful!",
            "Warning the owner has no use, even if I'd love to.",
        )
        .await;
    }

    Ok(())
}

#[group("Modtools")]
#[commands(purge, kick, ban, warn)]
#[summary = "Commands for moderators and admins of a server."]
#[only_in("guilds")]
struct ModTools;
