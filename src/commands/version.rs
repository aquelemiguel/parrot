use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
};
use std::env;

use crate::utils::send_simple_message;

const RELEASES_LINK: &str = "https://github.com/aquelemiguel/parrot/releases";

#[command]
async fn version(ctx: &Context, msg: &Message) -> CommandResult {
    let current = env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "Unknown".to_string());
    let current = format!("Version [{}]({}/tag/v{})", current, RELEASES_LINK, current);
    let latest = format!("Find the latest version [here]({}/latest)", RELEASES_LINK);
    let message = format!("{}\n{}", current, latest);
    send_simple_message(&ctx.http, msg, &message).await
}
