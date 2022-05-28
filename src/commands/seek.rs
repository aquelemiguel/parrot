use crate::{
    errors::{verify, ParrotError},
    messaging::Response,
    strings::{FAIL_MINUTES_PARSING, FAIL_SECONDS_PARSING},
    utils::create_response_,
};
use serenity::{
    client::Context, model::interactions::application_command::ApplicationCommandInteraction,
};
use std::time::Duration;

pub async fn seek(
    ctx: &Context,
    interaction: &mut ApplicationCommandInteraction,
) -> Result<(), ParrotError> {
    let guild_id = interaction.guild_id.unwrap();
    let manager = songbird::get(ctx).await.unwrap();
    let call = manager.get(guild_id).unwrap();

    let args = interaction.data.options.clone();
    let seek_time = args.first().unwrap().value.as_ref().unwrap();

    let timestamp_str = seek_time.as_str().unwrap();
    let mut units_iter = timestamp_str.split(':');

    let minutes = units_iter.next().and_then(|c| c.parse::<u64>().ok());
    let minutes = verify(minutes, ParrotError::Other(FAIL_MINUTES_PARSING))?;

    let seconds = units_iter.next().and_then(|c| c.parse::<u64>().ok());
    let seconds = verify(seconds, ParrotError::Other(FAIL_SECONDS_PARSING))?;

    let timestamp = minutes * 60 + seconds;

    let handler = call.lock().await;
    let track = handler
        .queue()
        .current()
        .ok_or(ParrotError::NothingPlaying)?;
    drop(handler);

    track.seek_time(Duration::from_secs(timestamp)).unwrap();

    create_response_(
        &ctx.http,
        interaction,
        Response::Seek {
            timestamp: timestamp_str.to_owned(),
        },
    )
    .await
}
