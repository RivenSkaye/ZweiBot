use log;
use serenity::{
    framework::standard::{
        macros::{command, group},
        Args, CommandResult,
    },
    model::prelude::*,
    prelude::*,
};
use std::collections::HashSet;

use crate::{
    dbx::{self, ZweiDbConn},
    send_err, send_err_titled, send_ok,
};

#[command("add")]
#[aliases("create", "+")]
#[only_in("guilds")]
#[required_permissions("MANAGE_GUILD")]
#[description = "Lets me notify people of certain tags when they subscribe to them. Tags are always a single word, so use dashes or underscores to avoid naming conflicts."]
#[help_available(true)]
async fn add_tags(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.is_empty() {
        return send_err(ctx, msg, "I can't add tags without an actual tag to add.").await;
    }
    let guild_id = msg.guild_id.unwrap().0;
    let mut ok_count: usize = args.len();
    let mut ok_list: String = String::with_capacity(args.message().len() + (args.len() * 3));
    let mut err_tags: Vec<String> = Vec::with_capacity(args.len());
    {
        let botdata = ctx.data.read().await;
        let conn = match botdata.get::<ZweiDbConn>() {
            Some(conn) => conn,
            _ => {
                log::error!("Failed to acquire database connection object to add tags!");
                return send_err_titled(
                    ctx,
                    msg,
                    "Catastrophic failure",
                    "Could not acquire the database connection object.\nContact support if this keeps happening!"
                ).await;
            }
        };
        for tag in args.iter() {
            let tagstr: String = tag?;
            match dbx::add_tag(conn, guild_id, &tagstr.to_lowercase()).await {
                Ok(_) => ok_list.push_str(format!("\n+ {tagstr}").as_str()),
                _ => {
                    log::warn!("Failed to add tag {tagstr} for {guild_id}");
                    ok_count -= 1;
                    err_tags.push(tagstr);
                }
            };
        }
    }
    match ok_count {
        1 => {
            send_ok(
                ctx,
                msg,
                "A new tag was added",
                format!("Added the following tag for subscribing to in this server:{ok_list}"),
            )
            .await?
        }
        2.. => {
            send_ok(
                ctx,
                msg,
                "New tags were added",
                format!("Added the following tags for subscribing to in this server:{ok_list}"),
            )
            .await?
        }
        _ => (),
    };
    match err_tags.len() {
        2.. => {
            send_err_titled(
                ctx,
                msg,
                "Tags already registered",
                format!(
                    "The following tags were already registered on this server:\n+ {}",
                    err_tags.join("\n+ ")
                ),
            )
            .await
        }
        1 => {
            send_err_titled(
                ctx,
                msg,
                "Tag already registered",
                format!(
                    "{} was already registered for this server.",
                    err_tags.get(0).unwrap()
                ),
            )
            .await
        }
        _ => Ok(()),
    }
}

#[command("remove")]
#[only_in("guilds")]
#[required_permissions("MANAGE_GUILD")]
#[description = "Makes me stop using this tag here, and unsubscribes all users from them."]
#[help_available(true)]
async fn remove_tags(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.is_empty() {
        return send_err(
            ctx,
            msg,
            "I can't remove tags without the actual tag to remove.",
        )
        .await;
    }
    let guild_id = msg.guild_id.unwrap().0;
    let mut ok_list: String = String::with_capacity(args.message().len() + (args.len() * 3));
    let mut err_tags: Vec<String> = Vec::with_capacity(args.len());
    {
        let botdata = ctx.data.read().await;
        let conn = match botdata.get::<ZweiDbConn>() {
            Some(conn) => conn,
            _ => {
                log::error!("Failed to acquire database connection object to remove tags!");
                return send_err_titled(
                    ctx,
                    msg,
                    "Catastrophic failure",
                    "Could not acquire the database connection object.\nContact support if this keeps happening!"
                ).await;
            }
        };
        for tag in args.iter() {
            let tagstr: String = tag?;
            match dbx::remove_tag(conn, guild_id, &tagstr.to_lowercase()).await {
                Ok(_) => ok_list.push_str(format!("\n+ {}", tagstr).as_str()),
                _ => {
                    log::warn!("Failed to remove tag {tagstr} for {guild_id}");
                    err_tags.push(tagstr)
                }
            };
        }
    }
    match ok_list.len() {
        1 => {
            send_ok(
                ctx,
                msg,
                "Tag removed",
                format!("This server can no longer use{ok_list}"),
            )
            .await?
        }
        2.. => {
            send_ok(
                ctx,
                msg,
                "Tags removed",
                format!("This server will no longer have the following tags:{ok_list}"),
            )
            .await?
        }
        _ => (),
    };
    match err_tags.len() {
        1 => {
            send_err_titled(
                ctx,
                msg,
                "Tag not found!",
                format!(
                    "{} wasn't registered for this server.",
                    err_tags.get(0).unwrap()
                ),
            )
            .await
        }
        2.. => {
            send_err_titled(
                ctx,
                msg,
                "Tags not found!",
                format!(
                    "This server didn't even have these:\n+ {}",
                    err_tags.join("\n+ ")
                ),
            )
            .await
        }
        _ => Ok(()),
    }
}

#[command("list")]
#[only_in("guilds")]
#[min_args(0)]
#[max_args(0)]
#[description = "Lets me know you want to see all tags available in the server."]
#[help_available(true)]
async fn list_tags(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap().0;
    {
        let botdata = ctx.data.read().await;
        let conn = match botdata.get::<ZweiDbConn>() {
            Some(conn) => conn,
            _ => {
                log::error!("Failed to acquire database connection object to list tags!");
                return send_err_titled(
                    ctx,
                    msg,
                    "Catastrophic failure",
                    "Could not acquire the database connection object.\nContact support if this keeps happening!"
                ).await;
            }
        };
        let tags = dbx::get_server_tags(conn, guild_id).await?;
        send_ok(
            ctx,
            msg,
            "Tags for this server",
            format!("+ {}", tags.join("\n+ ")),
        )
        .await
    }
}

#[command("sub")]
#[only_in("guilds")]
#[aliases("subscribe")]
#[description = "I'll give you a poke if you tell me the tags you're interested in."]
#[help_available(true)]
async fn subscribe(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.is_empty() {
        return send_err(
            ctx,
            msg,
            "Did you forget to add actual tags to subscribe to?",
        )
        .await;
    }
    let guild_id = msg.guild_id.unwrap().0;
    let auth = msg.author.id.0;
    let mut ok_list: String = String::with_capacity(args.message().len() + (args.len() * 3));
    let mut err_list = Vec::with_capacity(args.len());
    {
        let botdata = ctx.data.read().await;
        let conn = match botdata.get::<ZweiDbConn>() {
            Some(conn) => conn,
            _ => {
                log::error!("Failed to acquire database connection object to add subscription!");
                return send_err_titled(
                    ctx,
                    msg,
                    "Catastrophic failure",
                    "Could not acquire the database connection object.\nContact support if this keeps happening!"
                ).await;
            }
        };
        for tag in args.iter() {
            let tagstr: String = tag?;
            let res = dbx::sub_to(conn, guild_id, &tagstr, auth);
            match res.await {
                Ok(_) => ok_list.push_str(format!("\n+ {}", tagstr).as_str()),
                _ => {
                    log::warn!("Could not subscribe {auth} to {tagstr} in {guild_id}");
                    err_list.push(tagstr)
                }
            };
        }
    }
    match ok_list.len() {
        1.. => {
            send_ok(
                ctx,
                msg,
                "Subscribed successfully!",
                format!("You are now subscribed to:{ok_list}"),
            )
            .await?;
        }
        _ => {
            ();
        }
    };
    match err_list.len() {
        1.. => {
            send_err_titled(
                ctx,
                msg,
                "Subscription failed",
                format!(
                    "The following tags could not be found in this server:\n+ {}",
                    err_list.join("\n+ ")
                ),
            )
            .await
        }
        _ => Ok(()),
    }
}

#[command("unsub")]
#[only_in("guilds")]
#[aliases("unsubscribe")]
#[description = "Lets me know you don't want to be pinged for certain tags anymore."]
#[help_available(true)]
async fn unsubscribe(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.is_empty() {
        return send_err(
            ctx,
            msg,
            "Did you forget to add actual tags to unsubscribe from?",
        )
        .await;
    }
    let guild_id = msg.guild_id.unwrap().0;
    let auth = msg.author.id.0;
    let mut ok_list: String = String::with_capacity(args.message().len() + (args.len() * 3));
    let mut err_list = Vec::with_capacity(args.len());
    {
        let botdata = ctx.data.read().await;
        let conn = match botdata.get::<ZweiDbConn>() {
            Some(conn) => conn,
            _ => {
                log::error!("Failed to acquire database connection object to unsubscribe user!");
                return send_err_titled(
                    ctx,
                    msg,
                    "Catastrophic failure",
                    "Could not acquire the database connection object.\nContact support if this keeps happening!"
                ).await;
            }
        };
        for tag in args.iter() {
            let tagstr: String = tag?;
            let res = dbx::unsub(conn, guild_id, &tagstr, auth);
            match res.await {
                Ok(_) => ok_list.push_str(format!("\n+ {}", tagstr).as_str()),
                _ => {
                    log::warn!("Couldn't unsibscribe {auth} from {tagstr} in {guild_id}");
                    err_list.push(tagstr)
                }
            };
        }
    }
    match ok_list.len() {
        1.. => {
            send_ok(
                ctx,
                msg,
                "Unsubscribed successfully!",
                format!("You are no longer subscribed to:{ok_list}"),
            )
            .await?;
        }
        _ => {
            ();
        }
    };
    match err_list.len() {
        1.. => {
            send_err_titled(
                ctx,
                msg,
                "Unsubscribing failed",
                format!("You were not subscribed to following tags, or they could not be found in this server:\n+ {}", err_list.join("\n+ ")),
            )
            .await
        }
        _ => Ok(()),
    }
}

#[command("subs")]
#[only_in("guilds")]
#[aliases("subscriptions", "pongs")]
#[description = "I'll tell you what you're currently subscribed to."]
#[help_available(true)]
async fn list_subs(ctx: &Context, msg: &Message) -> CommandResult {
    let uid = msg.author.id.0;
    let guild_id = msg.guild_id.unwrap().0;
    let tags;
    {
        let botdata = ctx.data.read().await;
        let conn = match botdata.get::<ZweiDbConn>() {
            Some(conn) => conn,
            _ => {
                log::error!("Failed to acquire database connection object to list subscriptions!");
                return send_err_titled(
                    ctx,
                    msg,
                    "Catastrophic failure",
                    "Could not acquire the database connection object.\nContact support if this keeps happening!"
                ).await;
            }
        };
        tags = dbx::usersubs(conn, guild_id, uid).await?;
    }
    match tags.len() {
        1.. => {
            send_ok(
                ctx,
                msg,
                "Your subcriptions",
                format!(
                    "For this server, you are currently subscribed to:\n+ {}",
                    tags.join("\n+ ")
                ),
            )
            .await
        }
        _ => {
            log::warn!("Could not list subsctiptions for {uid} in {guild_id}");
            send_err_titled(
                ctx,
                msg,
                "No subcriptions found",
                "You are currently not subscribed to any tags in this server.",
            )
            .await
        }
    }
}

#[command("ping")]
#[only_in("guilds")]
#[description = "I'll tell everyone who wants to know that this tag was used."]
async fn ping_all_subbers(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if args.is_empty() {
        return send_err_titled(
            ctx,
            msg,
            "No tags",
            "I need at least one tag to notify people for.",
        )
        .await;
    }
    if args.message().chars().count() > 750 {
        return send_err_titled(
            ctx,
            msg,
            "Too many tags!",
            "Please limit the used tags to a maximum of 750 characters.\nThis isn't even healthy anymore!"
        ).await;
    }
    let tags: Vec<String> = args
        .message()
        .replace(", ", " ")
        .split(" ")
        .map(|a| String::from(a))
        .collect::<HashSet<String>>()
        .iter()
        .map(|s| s.to_owned())
        .collect();
    let guild_id = msg.guild_id.unwrap().0;
    let mut users: HashSet<u64> = HashSet::new();
    let mut failed: Vec<String> = Vec::new();
    let mut tagmsg = String::new();
    {
        let botdata = ctx.data.read().await;
        let conn = match botdata.get::<ZweiDbConn>() {
            Some(conn) => conn,
            _ => {
                log::error!("Failed to acquire database connection object to tag users!");
                return send_err_titled(
                    ctx,
                    msg,
                    "Catastrophic failure",
                    "Could not acquire the database connection object.\nContact support if this keeps happening!"
                ).await;
            }
        };
        let tagcount = dbx::are_tags_in_server(conn, guild_id, &tags);
        let valid = match tagcount.await {
            Ok(c) => c > 0,
            Err(e) => {
                log::warn!(
                    "Couldn't find any tags for {guild_id} matching `{}`",
                    tags.join(", ")
                );
                send_err(ctx, msg, format!("{e}")).await?;
                false
            }
        };
        if !valid {
            return send_err_titled(
                ctx,
                msg,
                "No matching tags found!",
                "None of what you just said makes any sense to me. Try checking `tag list` for what's available."
            ).await;
        }
        for tag in &tags {
            if tag == "" || tag == " " {
                continue;
            }
            let subs = dbx::get_subbers(conn, guild_id, &tag);
            match subs.await {
                Ok(s) => {
                    users.extend(s);
                }
                Err(_) => failed.push(tag.clone()),
            };
        }
        tagmsg.push_str(tags.join(", ").as_str());
    }
    if failed.len() > 0 {
        log::warn!(
            "Failed to find tags for {guild_id}: `{}`",
            failed.join(", ")
        );
        send_err_titled(
            ctx,
            msg,
            "Tags not found",
            format!(
                "The following tags do not exist:\n+ {}",
                failed.join("\n+ ")
            ),
        )
        .await?;
    }
    if users.len() == 0 {
        log::warn!(
            "Failed to find sunscribed users for {guild_id}: `{}`",
            failed.join(", ")
        );
        return send_err_titled(
            ctx,
            msg,
            "No users subscribed",
            "One or more tags exist, but no users in this server are subscribed to any of them.",
        )
        .await;
    } else {
        let mut paginated: Vec<String> = Vec::new();
        let initial = tagmsg.chars().count();
        let mut charcount: usize = initial;
        let mut pingmsg = String::with_capacity(2000);
        pingmsg.push_str(&tagmsg);
        for user in users {
            // A Discord ID is 18 chars long. A user ping (<@ID>) is 21
            // A space adds one, for 22 chars added.
            // Message limit is 2k. So 2k - 22 = 1978.
            // so make sure we can actually fit those 22 characters in the message
            // if not, push a clone of it into the vec up there.
            // Then send all messages.
            if charcount > 1978 {
                paginated.push(pingmsg.clone());
                pingmsg.clear();
                pingmsg.push_str(&tagmsg);
                charcount = initial;
            }
            pingmsg.push_str(format!(" <@{user}>").as_str());
            charcount += 22;
        }
        paginated.push(pingmsg);
        for page in paginated {
            msg.channel_id
                .send_message(ctx, |mes| mes.content(page))
                .await?;
        }
    }
    Ok(())
}

#[group("Tag")]
#[commands(
    add_tags,
    remove_tags,
    list_tags,
    subscribe,
    unsubscribe,
    list_subs,
    ping_all_subbers
)]
#[summary = "Tag subscription for easily pinging the people interested in certain subjects. Tags are case-insensitive. Provide only tags to ping subscribed users."]
#[prefixes("tag")]
#[help_available(true)]
#[default_command(ping_all_subbers)]
struct Tag;
