use serenity::client::Context;
use serenity::model::interactions::application_command::ApplicationCommandInteraction;
use serenity::prelude::SerenityError;
use std::time::Duration;

use crate::strings::{
    MISSING_TIMESTAMP, NO_VOICE_CONNECTION, QUEUE_IS_EMPTY, TIMESTAMP_PARSING_FAILED,
};
use crate::utils::create_response;

pub async fn seek(
    ctx: &Context,
    interaction: &mut ApplicationCommandInteraction,
) -> Result<(), SerenityError> {
    let guild_id = interaction.guild_id.unwrap();
    let manager = songbird::get(ctx).await.unwrap();

    let call = match manager.get(guild_id) {
        Some(call) => call,
        None => return create_response(&ctx.http, interaction, NO_VOICE_CONNECTION).await,
    };

    let args = interaction.data.options.clone();

    let seek_time = match args.first() {
        Some(t) if t.value.is_some() => t.value.as_ref().unwrap(),
        _ => return create_response(&ctx.http, interaction, MISSING_TIMESTAMP).await, // TODO: Possibly delete this
    };

    let timestamp = seek_time.as_str().unwrap();
    let mut units_iter = timestamp.split(':');

    let (minutes, seconds) = (
        units_iter
            .next()
            .map(|token| token.parse::<u64>().ok())
            .flatten(),
        units_iter
            .next()
            .map(|token| token.parse::<u64>().ok())
            .flatten(),
    );

    if minutes.is_none() || seconds.is_none() {
        return create_response(&ctx.http, interaction, TIMESTAMP_PARSING_FAILED).await;
    }

    let timestamp = minutes.unwrap() * 60 + seconds.unwrap();

    let handler = call.lock().await;
    let track = match handler.queue().current() {
        Some(track) => track,
        None => return create_response(&ctx.http, interaction, QUEUE_IS_EMPTY).await,
    };
    drop(handler);

    track.seek_time(Duration::from_secs(timestamp)).unwrap();

    create_response(
        &ctx.http,
        interaction,
        &format!("Seeked current track to **{}**!", seek_time),
    )
    .await
}
