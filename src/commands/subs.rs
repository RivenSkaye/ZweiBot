use serenity::{
    framework::standard::{
        macros::{command, group},
        Args, CommandResult,
    },
    model::{
        id::{GuildId, MessageId, UserId},
        prelude::*,
    },
    prelude::*,
};

use crate::{db::{self, ZweiDbConn}, send_err, send_err_titled, send_ok};

#[command("add")]
#[only_in("guilds")]
#[required_permissions("MANAGE_GUILD")]
#[summary = "Adds one or more tags for people in this server to subscribe to."]
async fn add_tags(ctx: &Context, msg: &Message, args: Args) {
    if args.is_empty() {
        return send_err(ctx, msg, "I can't add tags without the actual tag to add.");
    }
    let botdata = ctx.data.read().await;
    if let Some(conn) = botdata.get::<ZweiDbConn>() {
        let mut ok_msg: String = String::from(match args.len() {
            1 => "Added the following tag for subscribing to in this server:",
            _ => "Added the following tags for subscribing to in this server:"
        })
        for tag in args.iter() {
            let tagstr = String::from(tag)
            match db::add_tag(&conn, msg.guild(ctx).await?, &tagstr) {
                Ok => ok_msg.push_str(format!("\n- {tagstr}")),
                Err => send_err(ctx, msg, "The tag {tagstr} was already registered!")?
            }
        }
        return send_ok(ctx, msg, "New tags were added!", ok_msg);
    }
    return send_err_titled(
        ctx,
        msg,
        "Catastrophic failure",
        "Could not acquire the database connection object.\nContact support if this keeps happening!"
    );
}

#[group("Tagging")]
#[commands(add_tags)]
#[summary = "Tag subscription module for easily pinging the people interested in certain subjects."]
#[prefixes("tag")]
#[help_available]
struct Tagging;
