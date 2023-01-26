use chrono::{Timelike, Utc};
use log;
use serenity::{
    framework::standard::{
        macros::{command, group},
        Args, CommandResult,
    },
    futures::future,
    model::prelude::*,
    prelude::*,
};
use tokio::time::{sleep, Duration};

use crate::{
    dbx, get_name, get_prefix, send_err, send_err_titled, send_ok, ShardManagerContainer, ZweiData,
    ZweiDbConn, ZweiOwners, ZweiPrefixes,
};

#[command]
#[owners_only]
#[max_args(1)]
#[aliases("shutdown", "panic", "die", "sleep")]
#[description = "Stops me in my tracks. Optionally takes a time in seconds to wait, defaults to 1 second."]
#[example = "5"]
#[example = ""]
#[help_available]
async fn exit(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    args.trimmed();
    let time: u64 = args.parse::<u64>().unwrap_or(1);
    log::info!(
        "Received a shutdown request at UTC {} with {time}s timeout",
        Utc::now()
    );

    if let Some(manager) = ctx.data.read().await.get::<ShardManagerContainer>() {
        send_ok(
            ctx,
            msg,
            "Shutting down",
            format!(
                "I'm going down in {}.",
                if time >= 60 {
                    let secs = time % 60;
                    let mins = (time - secs) / 60;
                    format!(
                        "{mins} minute{} and {secs} second{}",
                        if mins != 1 { "s" } else { "" },
                        if secs != 1 { "s" } else { "" }
                    )
                } else {
                    format!("{time} second{}", if time != 1 { "s" } else { "" })
                }
            ),
        )
        .await?;
        sleep(Duration::from_secs(time)).await;
        log::info!("SYSTEM HALT");
        manager.lock().await.shutdown_all().await;
    } else {
        send_err(
            ctx,
            msg,
            "I've lost control of Kuro, I'm stopping RIGHT NOW!",
        )
        .await?;
        log::error!("Could not acquire shard manager, performing an unclean exit IMMEDIATELY!");
        ctx.data
            .read()
            .await
            .get::<ZweiDbConn>()
            .expect("Couldn't acquire context manager or database connection.")
            .close()
            .await;
        panic!("Couldn't get the context manager from bot code!")
    }
    Ok(())
}

#[command]
#[description = "Displays how long I've been roaming Lost Blue since ~~`on_ready` event fired~~ I woke up."]
async fn uptime(ctx: &Context, msg: &Message) -> CommandResult {
    if let Some(lifetime) = ctx.data.read().await.get::<ZweiData>() {
        let starttime = &lifetime["Init"];
        let diff = Utc::now().timestamp() - *starttime;
        let secs = diff % 60;
        let mins = (diff % 3600) / 60;
        let hours = diff / 3600;
        send_ok(
            ctx,
            msg,
            "Bot uptime",
            format!("I've been roaming for {hours} hours, {mins} minutes and {secs} seconds."),
        )
        .await
    } else {
        log::error!("Unable to retrieve ZweiData object!");
        send_err(
            ctx,
            msg,
            "I've been in Lost Blue for so long that I can't even remember when I got here...",
        )
        .await
    }
}

#[command]
#[description = "Makes me check the current UTC time according to UNIX timestamps."]
#[help_available]
async fn now(ctx: &Context, msg: &Message) -> CommandResult {
    let now = Utc::now();
    let hours = now.hour();
    let mins = now.minute();
    let secs = now.second();
    let difftxt = format!("{hours:02}:{mins:02}:{secs:02}");
    send_ok(ctx, msg, "Current UTC time", difftxt).await
}

#[command]
#[description = "I think you should know these people, they're the ones that keep me going!"]
#[help_available]
#[aliases("credits", "creators")]
async fn owners(ctx: &Context, msg: &Message) -> CommandResult {
    let botdata = ctx.data.read().await;
    let owner_names = future::try_join_all(
        botdata
            .get::<ZweiOwners>()
            .into_iter()
            .flatten()
            .copied()
            .map(|id| get_name(msg, ctx, id)),
    )
    .await?;

    log::trace!("Earning street cred");

    send_ok(
        ctx,
        msg,
        "Credits",
        format!(
            "These are the wonderful people who are making sure Kuro and I can keep going!\n- {}",
            owner_names.join("\n- ")
        ),
    )
    .await
}

#[command]
#[description = "This is how you can get me to listen to requests."]
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
#[description = "Changes the arcane symbol that notifies me of commands."]
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
                log::error!(
                    "Couldn't acquire dayabase connection to change prefix to {pfx} for {guild}!"
                );
                let etxt = "Something went wrong requesting the database connection!";
                send_err_titled(ctx, msg, "Change prefix", etxt).await?;
                return Err(etxt.into());
            }
        };

        match dbx::set_prefix(conn, guild, pfx).await? {
            1.. => (),
            _ => {
                log::error!("Couldn't update prefix for {guild} to {pfx}!");
                return send_err_titled(ctx, msg, "Change prefix", "Couldn't update the prefix!")
                    .await;
            }
        };
    }

    {
        if let Some(data) = ctx.data.write().await.get_mut::<ZweiPrefixes>() {
            data.insert(guild, pfx.to_owned());
        } else {
            log::error!("Unable to update prefix cache!");
            return send_err_titled(
                ctx,
                msg,
                "Change prefix",
                "Can't update the server prefix in the cache!",
            )
            .await;
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
#[description = "Makes me go back to listening to the default prefix."]
#[only_in("guilds")]
#[required_permissions("MANAGE_GUILD")]
async fn clear(ctx: &Context, msg: &Message) -> CommandResult {
    let guild: u64 = msg.guild_id.unwrap().0;

    {
        let botdata = ctx.data.read().await;
        if let Some(conn) = botdata.get::<ZweiDbConn>() {
            let res = dbx::remove_prefix(conn, guild).await?;
            match res {
                1 => (),
                0 => {
                    let etxt = "There was no custom prefix stored for this server.";
                    send_err_titled(ctx, msg, "Clear prefix", etxt).await?;
                    Err(String::from(etxt))?
                }
                n => {
                    log::warn!("Removed {n} prefixes for guild {guild}...");
                    let etxt = "Prefix change affected multiple rows...";
                    send_err_titled(ctx, msg, "Clear prefix", etxt).await?;
                    ()
                }
            };
        } else {
            log::error!("Could not acquire database connection to clear prefix for {guild}");
            let etxt = "Something went wrong requesting the database connection!";
            send_err_titled(ctx, msg, "Clear prefix", etxt).await?
        }
    }

    {
        let mut wbd = ctx.data.write().await;
        if let Some(data) = wbd.get_mut::<ZweiPrefixes>() {
            data.remove(&guild);
        } else {
            log::error!("Can't update the prefeix cache!");
            let etxt = "Can't update the cache!";
            send_err_titled(ctx, msg, "Clear prefix", etxt).await?
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
#[summary = "Miscellaneous commands for bot information."]
struct Misc;

#[group("Prefix")]
#[commands(get, set, clear)]
#[summary = "Change the current prefix, or display it if no extra arguments are given."]
#[prefixes("prefix")]
#[default_command(set)]
#[help_available]
struct Prefix;
