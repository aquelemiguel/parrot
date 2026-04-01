use crate::{
    errors::ParrotError, messaging::message::ParrotMessage, messaging::messages::FAIL_LOOP,
    utils::create_response,
};
use serenity::{all::CommandInteraction, client::Context};
use songbird::tracks::{LoopState, TrackHandle};

pub async fn repeat(
    ctx: &Context,
    interaction: &mut CommandInteraction,
) -> Result<(), ParrotError> {
    let guild_id = interaction.guild_id.ok_or(ParrotError::Other(
        "This command can only be used in a server",
    ))?;
    let manager = songbird::get(ctx)
        .await
        .ok_or(ParrotError::Other("Voice manager not configured"))?;
    let call = manager.get(guild_id).ok_or(ParrotError::NotConnected)?;

    let handler = call.lock().await;
    let track = handler
        .queue()
        .current()
        .ok_or(ParrotError::NothingPlaying)?;

    let track_info = track
        .get_info()
        .await
        .map_err(|_| ParrotError::Other("Failed to get track info"))?;
    let was_looping = track_info.loops == LoopState::Infinite;
    let toggler = if was_looping {
        TrackHandle::disable_loop
    } else {
        TrackHandle::enable_loop
    };

    match toggler(&track) {
        Ok(_) if was_looping => {
            create_response(&ctx.http, interaction, ParrotMessage::LoopDisable).await
        }
        Ok(_) if !was_looping => {
            create_response(&ctx.http, interaction, ParrotMessage::LoopEnable).await
        }
        _ => Err(ParrotError::Other(FAIL_LOOP)),
    }
}
