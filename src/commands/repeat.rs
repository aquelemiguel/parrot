use crate::{
    errors::ParrotError, messaging::Response, strings::FAIL_LOOP, utils::create_response_,
};
use serenity::{
    client::Context, model::interactions::application_command::ApplicationCommandInteraction,
};
use songbird::tracks::{LoopState, TrackHandle};

pub async fn repeat(
    ctx: &Context,
    interaction: &mut ApplicationCommandInteraction,
) -> Result<(), ParrotError> {
    let guild_id = interaction.guild_id.unwrap();
    let manager = songbird::get(ctx).await.unwrap();
    let call = manager.get(guild_id).unwrap();

    let handler = call.lock().await;
    let track = handler.queue().current().unwrap();

    let was_looping = track.get_info().await.unwrap().loops == LoopState::Infinite;
    let toggler = if was_looping {
        TrackHandle::disable_loop
    } else {
        TrackHandle::enable_loop
    };

    match toggler(&track) {
        Ok(_) if was_looping => {
            create_response_(&ctx.http, interaction, Response::LoopDisabled).await
        }
        Ok(_) if !was_looping => {
            create_response_(&ctx.http, interaction, Response::LoopEnabled).await
        }
        _ => Err(ParrotError::Other(FAIL_LOOP)),
    }
}
