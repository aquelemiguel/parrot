use crate::{
    errors::{verify, ParrotError},
    messaging::message::ParrotMessage,
    messaging::messages::{FAIL_MINUTES_PARSING, FAIL_SECONDS_PARSING},
    utils::create_response,
};
use serenity::{all::CommandInteraction, client::Context};
use std::time::Duration;

pub async fn seek(
    ctx: &Context,
    interaction: &mut CommandInteraction,
) -> Result<(), ParrotError> {
    let guild_id = interaction.guild_id.unwrap();
    let manager = songbird::get(ctx).await.unwrap();
    let call = manager.get(guild_id).unwrap();

    let args = interaction.data.options.clone();
    let timestamp_str = args
        .first()
        .and_then(|opt| opt.value.as_str())
        .unwrap_or("0:00");
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

    track.seek(Duration::from_secs(timestamp)).result().ok();

    create_response(
        &ctx.http,
        interaction,
        ParrotMessage::Seek {
            timestamp: timestamp_str.to_owned(),
        },
    )
    .await
}
