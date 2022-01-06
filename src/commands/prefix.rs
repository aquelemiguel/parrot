use std::fs;

use serde_json::json;
use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};

use crate::{
    strings::{MISSING_PREFIX, PREFIX_UPDATED},
    utils::{get_prefixes, merge_json, send_simple_message},
};

#[command]
async fn prefix(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let guild_id = msg.guild(&ctx.cache).await.unwrap().id;

    let mut prefixes = get_prefixes();
    let new_prefix = String::from(args.rewind().rest());

    if new_prefix.len() == 1 {
        let guild_new_prefix = json!({ guild_id.0.to_string(): new_prefix });

        merge_json(&mut prefixes, &guild_new_prefix);
        fs::write("prefixes.json", prefixes.to_string()).unwrap();

        send_simple_message(&ctx.http, msg, PREFIX_UPDATED).await
    } else {
        send_simple_message(&ctx.http, msg, MISSING_PREFIX).await
    }
}
