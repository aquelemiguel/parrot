use std::time::Duration;

use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult, Delimiter},
    model::channel::Message,
};

use crate::{
    strings::{MISSING_TIMESTAMP, NO_VOICE_CONNECTION, TIMESTAMP_PARSING_FAILED},
    utils::send_simple_message,
};

#[command]
async fn seek(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let guild_id = msg.guild(&ctx.cache).await.unwrap().id;
    let manager = songbird::get(ctx)
        .await
        .expect("Could not retrieve Songbird voice client");

    if let Some(call) = manager.get(guild_id) {
        let seek_time = match args.single::<String>() {
            Ok(t) => t,
            Err(_) => {
                send_simple_message(&ctx.http, msg, MISSING_TIMESTAMP).await;
                return Ok(());
            }
        };

        let mut timestamp = Args::new(&seek_time, &[Delimiter::Single(':')]);
        let (minutes, seconds) = (timestamp.single::<u64>(), timestamp.single::<u64>());

        if minutes.as_ref().and(seconds.as_ref()).is_err() {
            send_simple_message(&ctx.http, msg, TIMESTAMP_PARSING_FAILED).await;
            return Ok(());
        }

        let timestamp = minutes.unwrap() * 60 + seconds.unwrap();

        let handler = call.lock().await;
        let track = handler
            .queue()
            .current()
            .expect("Failed to fetch handle for current track");
        drop(handler);

        track
            .seek_time(Duration::from_secs(timestamp))
            .expect("Failed to seek on track");

        send_simple_message(
            &ctx.http,
            msg,
            &format!("Seeked current track to **{}**!", seek_time),
        )
        .await;
    } else {
        send_simple_message(&ctx.http, msg, NO_VOICE_CONNECTION).await;
    }

    Ok(())
}
