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
    db::{self, ZweiDbConn},
    send_err, send_err_titled, send_ok,
};

#[command("add")]
#[aliases("create", "+")]
#[only_in("guilds")]
#[required_permissions("MANAGE_GUILD")]
#[description = "Adds one or more tags for people in this server to subscribe to. Tags are always a single word, use dashes or underscores to avoid naming conflicts."]
#[help_available(true)]
async fn add_tags(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if args.is_empty() {
        return send_err(ctx, msg, "I can't add tags without the actual tag to add.").await;
    }
    let guild_id = msg.guild_id.unwrap().0;
    let mut ok_count: usize = args.len();
    let mut ok_list: String = String::with_capacity(args.message().len() + (args.len() * 3));
    let mut err_tags: Vec<String> = Vec::new();
    {
        let botdata = ctx.data.read().await;
        let conn = match botdata.get::<ZweiDbConn>() {
            Some(conn) => conn,
            _ => {
                return send_err_titled(
                    ctx,
                    msg,
                    "Catastrophic failure",
                    "Could not acquire the database connection object.\nContact support if this keeps happening!"
                ).await;
            }
        };
        let dbc = conn.lock().await;
        for tag in args.iter() {
            let tagstr: String = tag?;
            let res = db::add_tag(&dbc, guild_id, &tagstr.to_lowercase());
            match res {
                Ok(_) => ok_list.push_str(format!("\n+ {}", tagstr).as_str()),
                _ => {
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
    return match err_tags.len() {
        2.. => {
            let mut err_msg =
                String::from("The following tags were already registered on this server:");
            for e in err_tags {
                err_msg.push_str(format!("\n+ {}", e).as_str());
            }
            send_err_titled(ctx, msg, "Tags already registered", err_msg).await
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
    };
}

#[command("remove")]
#[only_in("guilds")]
#[required_permissions("MANAGE_GUILD")]
#[description = "Removes tags saved for this server."]
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
    let mut err_list = ok_list.clone();
    {
        let botdata = ctx.data.read().await;
        let conn = match botdata.get::<ZweiDbConn>() {
            Some(conn) => conn,
            _ => {
                return send_err_titled(
                    ctx,
                    msg,
                    "Catastrophic failure",
                    "Could not acquire the database connection object.\nContact support if this keeps happening!"
                ).await;
            }
        };
        let dbc = conn.lock().await;
        for tag in args.iter() {
            let tagstr: String = tag?;
            let res = db::remove_tag(&dbc, guild_id, &tagstr.to_lowercase());
            match res {
                Ok(_) => ok_list.push_str(format!("\n+ {}", tagstr).as_str()),
                _ => err_list.push_str(format!("\n+ {}", tagstr).as_str()),
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
    return match err_list.len() {
        1 => {
            send_err_titled(
                ctx,
                msg,
                "Tag not found!",
                format!("{err_list} wasn't registered for this server."),
            )
            .await
        }
        2.. => {
            send_err_titled(
                ctx,
                msg,
                "Tags not found!",
                format!("This server didn't even have{err_list}"),
            )
            .await
        }
        _ => Ok(()),
    };
}

#[command("list")]
#[only_in("guilds")]
#[min_args(0)]
#[max_args(0)]
#[description = "Lists all tags available for subscribing to in this server."]
#[help_available(true)]
async fn list_tags(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap().0;
    {
        let botdata = ctx.data.read().await;
        let conn = match botdata.get::<ZweiDbConn>() {
            Some(conn) => conn,
            _ => {
                return send_err_titled(
                    ctx,
                    msg,
                    "Catastrophic failure",
                    "Could not acquire the database connection object.\nContact support if this keeps happening!"
                ).await;
            }
        };
        let dbc = conn.lock().await;
        let tags = db::get_server_tags(&dbc, guild_id)?;
        send_ok(
            ctx,
            msg,
            "Tags for this server",
            format!("+ {}", tags.join("+ ")),
        )
        .await
    }
}

#[command("sub")]
#[only_in("guilds")]
#[aliases("subscribe")]
#[description = "Subscribe to one or more tags in this server."]
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
    let mut err_list = ok_list.clone();
    {
        let botdata = ctx.data.read().await;
        let conn = match botdata.get::<ZweiDbConn>() {
            Some(conn) => conn,
            _ => {
                return send_err_titled(
                    ctx,
                    msg,
                    "Catastrophic failure",
                    "Could not acquire the database connection object.\nContact support if this keeps happening!"
                ).await;
            }
        };
        let dbc = conn.lock().await;
        for tag in args.iter() {
            let tagstr: String = tag?;
            let res = db::sub_to(&dbc, guild_id, &tagstr, auth);
            match res {
                Ok(_) => ok_list.push_str(format!("\n+ {}", tagstr).as_str()),
                _ => err_list.push_str(format!("\n+ {}", tagstr).as_str()),
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
    return match err_list.len() {
        1.. => {
            send_err_titled(
                ctx,
                msg,
                "Subscription failed",
                format!("The following tags could not be found in this server:{err_list}"),
            )
            .await
        }
        _ => Ok(()),
    };
}

#[command("unsub")]
#[only_in("guilds")]
#[aliases("unsubscribe")]
#[description = "Unsubscribe from one or more tags in this server."]
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
    let mut err_list = ok_list.clone();
    {
        let botdata = ctx.data.read().await;
        let conn = match botdata.get::<ZweiDbConn>() {
            Some(conn) => conn,
            _ => {
                return send_err_titled(
                    ctx,
                    msg,
                    "Catastrophic failure",
                    "Could not acquire the database connection object.\nContact support if this keeps happening!"
                ).await;
            }
        };
        let dbc = conn.lock().await;
        for tag in args.iter() {
            let tagstr: String = tag?;
            let res = db::unsub(&dbc, guild_id, &tagstr, auth);
            match res {
                Ok(_) => ok_list.push_str(format!("\n+ {}", tagstr).as_str()),
                _ => err_list.push_str(format!("\n+ {}", tagstr).as_str()),
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
    return match err_list.len() {
        1.. => {
            send_err_titled(
                ctx,
                msg,
                "Unsubscribing failed",
                format!("You were not subscribed to following tags, or they could not be found in this server:{err_list}"),
            )
            .await
        }
        _ => Ok(()),
    };
}

#[command("subs")]
#[only_in("guilds")]
#[aliases("subscriptions", "pongs")]
#[description = "List all tags you're currently subscribed to."]
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
                return send_err_titled(
                    ctx,
                    msg,
                    "Catastrophic failure",
                    "Could not acquire the database connection object.\nContact support if this keeps happening!"
                ).await;
            }
        };
        let dbc = conn.lock().await;
        tags = db::usersubs(&dbc, guild_id, uid)?;
    }
    return match tags.len() {
        1.. => {
            send_ok(
                ctx,
                msg,
                "Your subcriptions",
                format!(
                    "For this server, you are currently subscribed to:\n+ {}",
                    tags.join("+ ")
                ),
            )
            .await
        }
        _ => {
            send_err_titled(
                ctx,
                msg,
                "No subcriptions found",
                "You are currently not subscribed to any tags in this server.",
            )
            .await
        }
    };
}

#[command("ping")]
#[only_in("guilds")]
#[description = "Notify all subscribed users of a relevant post."]
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
                return send_err_titled(
                    ctx,
                    msg,
                    "Catastrophic failure",
                    "Could not acquire the database connection object.\nContact support if this keeps happening!"
                ).await;
            }
        };
        let dbc = conn.lock().await;
        let tagcount = db::are_tags_in_server(&dbc, guild_id, &tags);
        let valid = match tagcount {
            Ok(c) => c > 0,
            Err(e) => {
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
            let subs = db::get_subbers(&dbc, guild_id, &tag);
            match subs {
                Ok(s) => {
                    users.extend(s);
                }
                Err(_) => failed.push(tag.clone()),
            };
        }
        tagmsg.push_str(tags.join(", ").as_str());
    }
    if failed.len() > 0 {
        send_err_titled(
            ctx,
            msg,
            "Tags not found",
            format!("The following tags do not exist: {}", failed.join("\n+ ")),
        )
        .await?;
    }
    if users.len() == 0 {
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
            // A Discord ID is 18 chars long. A user ping (<&ID>) is 21
            // Message limit is 2k. So 2k - 21 = 1979.
            // SPACES ARE FOR THE WEAK!
            // so make sure we can actually fit those 21 characters in the message
            // if not, push a clone of it into the vec up there.
            // Then send all messages.
            if charcount > 1979 {
                paginated.push(pingmsg.clone());
                pingmsg.clear();
                pingmsg.push_str(&tagmsg);
                charcount = initial;
            }
            pingmsg.push_str(format!(" <@{user}>").as_str());
            charcount += 21;
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
