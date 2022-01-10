use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
};

use crate::utils::send_simple_message;

const RELEASES_LINK: &str = "https://github.com/aquelemiguel/parrot/releases";

#[command]
#[aliases("v")]
async fn version(ctx: &Context, msg: &Message) -> CommandResult {
    let current = option_env!("CARGO_PKG_VERSION").unwrap_or_else(|| "Unknown");
    let current = format!("Version [{}]({}/tag/v{})", current, RELEASES_LINK, current);
    let latest = format!("Find the latest version [here]({}/latest)", RELEASES_LINK);
    let message = format!("{}\n{}", current, latest);
    send_simple_message(&ctx.http, msg, &message).await
}
