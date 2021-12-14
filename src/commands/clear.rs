use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
};

use crate::{
    strings::{AUTHOR_NOT_DJ, NO_VOICE_CONNECTION, QUEUE_IS_EMPTY},
    utils::{author_is_dj, send_simple_message},
};

#[command]
async fn clear(ctx: &Context, msg: &Message) -> CommandResult {
    if !author_is_dj(ctx, msg).await {
        send_simple_message(&ctx.http, msg, AUTHOR_NOT_DJ).await;
        return Ok(());
    } else {
        let guild_id = msg.guild(&ctx.cache).await.unwrap().id;
        let manager = songbird::get(ctx)
            .await
            .expect("Could not retrieve Songbird voice client");

        if let Some(call) = manager.get(guild_id) {
            let handler = call.lock().await;
            let queue = handler.queue().current_queue();
            drop(handler);

            if queue.is_empty() {
                send_simple_message(&ctx.http, msg, QUEUE_IS_EMPTY).await;
            } else {
                let handler = call.lock().await;
                handler.queue().modify_queue(|v| {
                    v.drain(1..);
                });
                drop(handler);

                send_simple_message(&ctx.http, msg, "Cleared!").await;
            }
        } else {
            send_simple_message(&ctx.http, msg, NO_VOICE_CONNECTION).await;
        }
    }
    Ok(())
}
