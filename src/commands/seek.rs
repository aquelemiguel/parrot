use crate::{
    errors::ParrotError,
    strings::{FAIL_TIMESTAMP_PARSING, NOTHING_IS_PLAYING, SEEKED},
    utils::create_response,
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

    let timestamp = seek_time.as_str().unwrap();
    let mut units_iter = timestamp.split(':');

    let (minutes, seconds) = (
        units_iter
            .next()
            .and_then(|token| token.parse::<u64>().ok()),
        units_iter
            .next()
            .and_then(|token| token.parse::<u64>().ok()),
    );

    if minutes.is_none() || seconds.is_none() {
        return create_response(&ctx.http, interaction, FAIL_TIMESTAMP_PARSING).await;
    }

    let timestamp = minutes.unwrap() * 60 + seconds.unwrap();

    let handler = call.lock().await;
    let track = match handler.queue().current() {
        Some(track) => track,
        None => return create_response(&ctx.http, interaction, NOTHING_IS_PLAYING).await,
    };
    drop(handler);

    track.seek_time(Duration::from_secs(timestamp)).unwrap();

    create_response(
        &ctx.http,
        interaction,
        &format!("{} **{}**!", SEEKED, seek_time),
    )
    .await
}
