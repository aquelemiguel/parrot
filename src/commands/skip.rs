use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
};

use crate::{strings::{NO_VOICE_CONNECTION, QUEUE_IS_EMPTY}, utils::send_simple_message};

#[command]
#[aliases("s")]
async fn skip(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild(&ctx.cache).await.unwrap().id;
    let manager = songbird::get(ctx).await.expect("Could not retrieve Songbird voice client");

    if let Some(call) = manager.get(guild_id) {
        let handler = call.lock().await;

        if handler.queue().is_empty() {
            send_simple_message(&ctx.http, msg, QUEUE_IS_EMPTY).await;
        }
        else if handler.queue().skip().is_ok() {
            send_simple_message(&ctx.http, msg, "Skipped!").await;
        }
    } else {
        send_simple_message(&ctx.http, msg, NO_VOICE_CONNECTION).await;
    }

    Ok(())
}
