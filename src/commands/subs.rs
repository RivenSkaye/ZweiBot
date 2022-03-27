use serenity::{
    framework::standard::{
        macros::{command, group},
        Args, CommandResult,
    },
    model::prelude::*,
    prelude::*,
};

use crate::{
    db::{self, ZweiDbConn},
    send_err, send_err_titled, send_ok,
};

#[command("add")]
#[only_in("guilds")]
#[required_permissions("MANAGE_GUILD")]
#[description = "Adds one or more tags for people in this server to subscribe to. Tags are always a single word, use dashes or underscores to avoid naming conflicts."]
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

#[group("Tagging")]
#[commands(add_tags, remove_tags, list_tags, subscribe, unsubscribe, list_subs)]
#[summary = "Tag subscription for easily pinging the people interested in certain subjects. Tags are case-insensitive. Provide only tags to ping subscribed users."]
#[prefixes("tag")]
#[help_available]
struct Tagging;
