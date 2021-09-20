use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
};

use crate::utils::send_simple_message;

#[command]
async fn pause(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild(&ctx.cache).await.unwrap().id;
    let manager = songbird::get(ctx).await.expect("").clone();

    if let Some(call) = manager.get(guild_id) {
        let handler = call.lock().await;

        if handler.queue().is_empty() {
            send_simple_message(&ctx.http, msg, "Queue is already empty!").await;
        }
        else if handler.queue().pause().is_ok() {
            send_simple_message(&ctx.http, msg, "Paused!").await;
        }
    } else {
        send_simple_message(&ctx.http, msg, "I'm not connected to any voice channel!").await;
    }

    Ok(())
}
