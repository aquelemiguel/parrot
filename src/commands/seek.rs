use std::time::Duration;

use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult, Delimiter},
    model::channel::Message,
};

use crate::{
    strings::{MISSING_TIMESTAMP, NO_VOICE_CONNECTION, QUEUE_IS_EMPTY, TIMESTAMP_PARSING_FAILED},
    utils::send_simple_message,
};

#[command]
async fn seek(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let guild_id = msg.guild(&ctx.cache).await.unwrap().id;
    let manager = songbird::get(ctx).await.unwrap();

    let call = match manager.get(guild_id) {
        Some(call) => call,
        None => return send_simple_message(&ctx.http, msg, NO_VOICE_CONNECTION).await,
    };

    let seek_time = match args.single::<String>() {
        Ok(t) => t,
        Err(_) => return send_simple_message(&ctx.http, msg, MISSING_TIMESTAMP).await,
    };

    let mut timestamp = Args::new(&seek_time, &[Delimiter::Single(':')]);
    let (minutes, seconds) = (timestamp.single::<u64>(), timestamp.single::<u64>());

    if minutes.as_ref().and(seconds.as_ref()).is_err() {
        return send_simple_message(&ctx.http, msg, TIMESTAMP_PARSING_FAILED).await;
    }

    let timestamp = minutes.unwrap() * 60 + seconds.unwrap();

    let handler = call.lock().await;
    let track = match handler.queue().current() {
        Some(track) => track,
        None => return send_simple_message(&ctx.http, msg, QUEUE_IS_EMPTY).await,
    };
    drop(handler);

    track.seek_time(Duration::from_secs(timestamp)).unwrap();

    return send_simple_message(
        &ctx.http,
        msg,
        &format!("Seeked current track to **{}**!", seek_time),
    )
    .await;
}
