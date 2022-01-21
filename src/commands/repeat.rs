use serenity::{
    client::Context, model::interactions::application_command::ApplicationCommandInteraction,
    prelude::SerenityError,
};
use songbird::tracks::{LoopState, TrackHandle};

use crate::{strings::NO_VOICE_CONNECTION, utils::create_response};

pub async fn repeat(
    ctx: &Context,
    interaction: &mut ApplicationCommandInteraction,
) -> Result<(), SerenityError> {
    let guild_id = interaction.guild_id.unwrap();
    let manager = songbird::get(ctx).await.unwrap();

    let call = match manager.get(guild_id) {
        Some(call) => call,
        None => return create_response(&ctx.http, interaction, NO_VOICE_CONNECTION).await,
    };

    let handler = call.lock().await;
    let track = handler.queue().current().unwrap();

    let was_looping = track.get_info().await.unwrap().loops == LoopState::Infinite;
    let toggler = if was_looping {
        TrackHandle::disable_loop
    } else {
        TrackHandle::enable_loop
    };

    match toggler(&track) {
        Ok(_) if was_looping => create_response(&ctx.http, interaction, "🔁  Disabled loop!").await,
        Ok(_) if !was_looping => create_response(&ctx.http, interaction, "🔁  Enabled loop!").await,
        _ => create_response(&ctx.http, interaction, "⚠️  Failed to toggle loop!").await,
    }
}
