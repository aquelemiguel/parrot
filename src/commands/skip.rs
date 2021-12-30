use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
};

use crate::{
    strings::{NO_VOICE_CONNECTION, QUEUE_IS_EMPTY},
    utils::send_simple_message,
};

#[command]
#[aliases("s")]
async fn skip(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild(&ctx.cache).await.unwrap().id;
    let manager = songbird::get(ctx)
        .await
        .unwrap();

    if let Some(call) = manager.get(guild_id) {
        let handler = call.lock().await;
        let queue = handler.queue();

        if queue.is_empty() {
            drop(handler);
            send_simple_message(&ctx.http, msg, QUEUE_IS_EMPTY).await;
        } else if queue.skip().is_ok() {
            drop(handler);
            send_simple_message(&ctx.http, msg, "⏭️ Skipped!").await;
        }
    } else {
        send_simple_message(&ctx.http, msg, NO_VOICE_CONNECTION).await;
    }

    Ok(())
}
