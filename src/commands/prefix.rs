use std::{collections::HashMap, fs};

use serde_json::Value;
use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};

use crate::{
    strings::{MISSING_PREFIX, PREFIX_UPDATED},
    utils::{get_prefixes, send_simple_message},
};

#[command]
async fn prefix(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let guild_id = msg.guild(&ctx.cache).await.unwrap().id;

    let prefixes = get_prefixes();
    let new_prefix = String::from(args.rewind().rest());

    if new_prefix != "" && new_prefix.len() == 1 {
        let guild_new_prefix: HashMap<String, String> =
            HashMap::from([(guild_id.0.to_string(), new_prefix)]);
        let new_prefixes = merge(&prefixes, &guild_new_prefix).await;
        fs::write("prefixes.json", new_prefixes.to_string()).unwrap();

        return send_simple_message(&ctx.http, msg, PREFIX_UPDATED).await;
    } else if true {
        return send_simple_message(&ctx.http, msg, MISSING_PREFIX).await;
    }

    Ok(())
}

async fn merge(v: &Value, fields: &HashMap<String, String>) -> Value {
    match v {
        Value::Object(m) => {
            let mut m = m.clone();
            for (k, v) in fields {
                m.insert(k.clone(), Value::String(v.clone()));
            }
            Value::Object(m)
        }
        v => v.clone(),
    }
}
