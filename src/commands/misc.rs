use chrono::Utc;
use serenity::{
    framework::standard::{
        macros::{command, group},
        Args, CommandResult,
    },
    model::prelude::*,
    prelude::*,
};
use tokio::time::{sleep, Duration};

use crate::{
    db, get_name, get_prefix, send_err, send_err_titled, send_ok, ShardManagerContainer, ZweiData,
    ZweiDbConn, ZweiOwners, ZweiPrefixes,
};

#[command]
#[owners_only]
#[max_args(1)]
#[aliases("shutdown", "panic", "die", "sleep")]
#[description = "Shuts down the bot, owner only. Optionally takes a time in seconds to wait, defaults to 1 second."]
#[example = "5"]
#[example = ""]
#[help_available]
async fn exit(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let cmd_name = "Shutting down";
    args.trimmed();
    let time: u64 = args.parse::<u64>().unwrap_or(1);
    args.advance();
    let botdata = ctx.data.read().await;

    if let Some(manager) = botdata.get::<ShardManagerContainer>() {
        send_ok(
            ctx,
            msg,
            cmd_name,
            format!("I'm taking a nap in {time} seconds."),
        )
        .await?;
        sleep(Duration::from_secs(time)).await;
        manager.lock().await.shutdown_all().await;
    } else {
        send_err(
            ctx,
            msg,
            "I've lost control of Kuro, I'm stopping RIGHT NOW!",
        )
        .await?;
        panic!("Couldn't get the context manager from bot code!")
    }
    Ok(())
}

#[command]
#[description = "Prints the bot's total running time since the `on_ready` event fired."]
async fn uptime(ctx: &Context, msg: &Message) -> CommandResult {
    let botdata = ctx.data.read().await;
    let cmd_title = "Bot uptime";
    if let Some(lifetime) = botdata.get::<ZweiData>() {
        let now = Utc::now().timestamp();
        let starttime = &lifetime["Init"];
        let diff = now - *starttime;
        let secs = diff % 60;
        let mins = (diff % 3600) / 60;
        let hours = diff / 3600;
        let difftxt = format!(
            "I've been running around for {hours} hours, {mins} minutes and {secs} seconds now.",
        );
        send_ok(ctx, msg, cmd_title, difftxt).await
    } else {
        send_err(
            ctx,
            msg,
            "I've been in Lost Blue for so long that I can't even remember when I got here...",
        )
        .await
    }
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
    let difftxt = format!("{hours:02}:{mins:02}:{secs:02}");
    send_ok(ctx, msg, "Current UTC time", difftxt).await
}

#[command]
#[description = "Provides information about the amazing people behind the bot."]
#[help_available]
#[aliases("credits", "creators")]
async fn owners(ctx: &Context, msg: &Message) -> CommandResult {
    let botdata = ctx.data.read().await;
    let owner_ids = botdata.get::<ZweiOwners>().into_iter().flatten().copied();

    let mut owner_names = vec![
        "These are the wonderful people who wrote me or were guinea pigs for testing!".to_owned(),
    ];
    for id in owner_ids {
        owner_names.push(get_name(msg, ctx, id).await?);
    }

    send_ok(ctx, msg, "Credits", owner_names.join("\n- ")).await
}

#[command]
#[description = "Check the prefix for the current context"]
#[help_available(false)]
async fn get(ctx: &Context, msg: &Message) -> CommandResult {
    send_ok(
        ctx,
        msg,
        "Prefix",
        format!(
            "I'm listening for {} in this server.",
            get_prefix(msg, ctx).await
        ),
    )
    .await
}

#[command]
#[description = "Change the guild prefix. Internally defers the command to get the current prefix if no arguments are given."]
#[min_args(0)]
#[max_args(1)]
#[only_in("guilds")]
#[required_permissions("MANAGE_GUILD")]
#[example = "z;"]
#[aliases("change")]
async fn set(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if args.is_empty() {
        return get(ctx, msg, args.clone()).await;
    }
    let guild: u64 = msg.guild_id.unwrap().0;
    let pfx = args.rest();
    {
        let botdata = ctx.data.read().await;
        let conn = match botdata.get::<ZweiDbConn>() {
            Some(conn) => conn,
            _ => {
                let etxt = "Something went wrong requesting the database connection!";
                send_err_titled(ctx, msg, "Change prefix", etxt).await?;
                return Err(etxt.into());
            }
        };
        let dbc = conn.lock().await;
        let res = db::set_prefix(&dbc, guild, pfx)?;
        match res {
            1.. => (),
            _ => {
                let etxt = "Couldn't update the prefix!";
                send_err_titled(ctx, msg, "Change prefix", etxt).await?;
                return Err(etxt.into());
            }
        };
    }

    {
        let mut wbd = ctx.data.write().await;
        if let Some(data) = wbd.get_mut::<ZweiPrefixes>() {
            data.insert(guild, pfx.to_owned());
        } else {
            let etxt = "Can't update the server prefix in the cache!";
            send_err_titled(ctx, msg, "Change prefix", etxt).await?;
            return Err(etxt.into());
        }
    }

    send_ok(
        ctx,
        msg,
        "Prefix changed",
        format!("From now on I'll respond to {pfx} here."),
    )
    .await
}

#[command]
#[description = "Clear the guild prefix and reset it to the bot's default."]
#[only_in("guilds")]
#[required_permissions("MANAGE_GUILD")]
async fn clear(ctx: &Context, msg: &Message) -> CommandResult {
    let guild: u64 = msg.guild_id.unwrap().0;
    {
        let botdata = ctx.data.read().await;
        if let Some(conn) = botdata.get::<ZweiDbConn>() {
            let dbc = conn.lock().await;
            let res = db::remove_prefix(&dbc, guild)?;
            match res {
                1 => (),
                0 => {
                    let etxt = "There was no custom prefix stored for this server.";
                    send_err_titled(ctx, msg, "Clear prefix", etxt).await?;
                    Err(String::from(etxt))?
                }
                _ => {
                    let etxt = "Prefix change affected multiple rows...";
                    send_err_titled(ctx, msg, "Clear prefix", etxt).await?;
                    ()
                }
            };
        } else {
            let etxt = "Something went wrong requesting the database connection!";
            send_err_titled(ctx, msg, "Clear prefix", etxt).await?;
            Err(String::from(etxt))?;
        }
    }

    {
        let mut wbd = ctx.data.write().await;
        if let Some(data) = wbd.get_mut::<ZweiPrefixes>() {
            data.remove(&guild);
        } else {
            let etxt = "Can't update the cache!";
            send_err_titled(ctx, msg, "Clear prefix", etxt).await?;
            Err(String::from(etxt))?
        }
    }

    send_ok(
        ctx,
        msg,
        "Prefix cleared",
        "From now on I'll respond to ; here.",
    )
    .await
}

#[group("Misc")]
#[commands(exit, uptime, now, owners)]
#[summary = "Miscellaneous commands for bot management and statistics."]
struct Misc;

#[group("Prefix")]
#[commands(get, set, clear)]
#[summary = "Change the current prefix, or display it if no extra arguments are given."]
#[prefixes("prefix")]
#[default_command(set)]
#[help_available]
struct Prefix;
