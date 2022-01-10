use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};

use crate::{
    strings::{MISSING_INDEX_QUEUE, NO_SONG_ON_INDEX, NO_VOICE_CONNECTION, QUEUE_IS_EMPTY},
    utils::send_simple_message,
};

#[command]
#[aliases("rm")]
async fn remove(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let guild_id = msg.guild(&ctx.cache).await.unwrap().id;
    let manager = songbird::get(ctx).await.unwrap();

    let call = match manager.get(guild_id) {
        Some(call) => call,
        None => return send_simple_message(&ctx.http, msg, NO_VOICE_CONNECTION).await,
    };

    let remove_index: usize = match args.single::<usize>() {
        Ok(t) => t,
        Err(_) => return send_simple_message(&ctx.http, msg, MISSING_INDEX_QUEUE).await,
    };

    let handler = call.lock().await;
    let queue = handler.queue().current_queue();

    if queue.is_empty() {
        send_simple_message(&ctx.http, msg, QUEUE_IS_EMPTY).await
    } else if queue.len() < remove_index + 1 {
        send_simple_message(&ctx.http, msg, NO_SONG_ON_INDEX).await
    } else if remove_index == 0 {
        send_simple_message(&ctx.http, msg, "Can't remove currently playing song!").await
    } else {
        handler.queue().modify_queue(|v| {
            v.remove(remove_index);
        });
        send_simple_message(&ctx.http, msg, &format!("Removed track #{}!", remove_index)).await
    }
}
