use std::sync::Arc;

use crate::{
    strings::{NO_VOICE_CONNECTION, QUEUE_IS_EMPTY},
    utils::{get_human_readable_timestamp, send_simple_message},
};
use serenity::{
    builder::CreateEmbedFooter,
    client::Context,
    framework::standard::{macros::command, CommandResult},
    http::Http,
    model::channel::Message,
};
use songbird::tracks::TrackHandle;

#[command]
#[aliases("np")]
pub async fn now_playing(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild(&ctx.cache).await.unwrap().id;
    let manager = songbird::get(ctx).await.unwrap();

    let call = match manager.get(guild_id) {
        Some(call) => call,
        None => return send_simple_message(&ctx.http, msg, NO_VOICE_CONNECTION).await,
    };

    let handler = call.lock().await;

    let track = match handler.queue().current() {
        Some(track) => track,
        None => return send_simple_message(&ctx.http, msg, QUEUE_IS_EMPTY).await,
    };

    send_now_playing_message(&ctx.http, msg, track).await
}

async fn send_now_playing_message(
    http: &Arc<Http>,
    msg: &Message,
    track: TrackHandle,
) -> CommandResult {
    let position = track.get_info().await?.position;
    let duration = track.metadata().duration.unwrap();
    let thumbnail = track.metadata().thumbnail.as_ref().unwrap();

    msg.channel_id
        .send_message(http, |m| {
            m.embed(|e| {
                e.title("Now playing");
                e.thumbnail(thumbnail);

                let title = track.metadata().title.as_ref().unwrap();
                let url = track.metadata().source_url.as_ref().unwrap();
                e.description(format!("[**{}**]({})", title, url));

                let mut footer = CreateEmbedFooter::default();
                let position_human = get_human_readable_timestamp(position);
                let duration_human = get_human_readable_timestamp(duration);

                footer.text(format!("{} / {}", position_human, duration_human));
                e.set_footer(footer)
            })
        })
        .await?;
    Ok(())
}
