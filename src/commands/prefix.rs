use std::fs;

use serde_json::{json, Value};
use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};

use crate::{
    strings::{FAILED_SAVE_PREFIXES, MISSING_PREFIX, PREFIX_UPDATED},
    utils::{get_prefixes, send_simple_message},
};

#[command]
async fn prefix(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let guild_id = msg.guild(&ctx.cache).await.unwrap().id;

    let mut prefixes = get_prefixes();
    let new_prefix = String::from(args.rewind().rest());

    if new_prefix.len() == 1 {
        let guild_new_prefix = json!({ guild_id.0.to_string(): new_prefix });

        merge(&mut prefixes, &guild_new_prefix);
        fs::write("prefixes.json", prefixes.to_string()).expect(FAILED_SAVE_PREFIXES);

        send_simple_message(&ctx.http, msg, PREFIX_UPDATED).await
    } else {
        send_simple_message(&ctx.http, msg, MISSING_PREFIX).await
    }
}

fn merge(a: &mut Value, b: &Value) {
    match (a, b) {
        (&mut Value::Object(ref mut a), &Value::Object(ref b)) => {
            for (k, v) in b {
                merge(a.entry(k.clone()).or_insert(Value::Null), v);
            }
        }
        (a, b) => {
            *a = b.clone();
        }
    }
}
