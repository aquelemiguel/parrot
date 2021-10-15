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
async fn resume(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild(&ctx.cache).await.unwrap().id;
    let manager = songbird::get(ctx)
        .await
        .expect("Could not retrieve Songbird voice client");

    if let Some(call) = manager.get(guild_id) {
        let handler = call.lock().await;
        let queue = handler.queue();

        if queue.is_empty() {
            drop(handler);
            send_simple_message(&ctx.http, msg, QUEUE_IS_EMPTY).await;
        } else if queue.resume().is_ok() {
            drop(handler);
            send_simple_message(&ctx.http, msg, "Resumed!").await;
        }
    } else {
        send_simple_message(&ctx.http, msg, NO_VOICE_CONNECTION).await;
    }

    Ok(())
}
