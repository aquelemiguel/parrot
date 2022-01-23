use serenity::{
    client::Context, model::interactions::application_command::ApplicationCommandInteraction,
    prelude::SerenityError,
};

use crate::{
    strings::{FAIL_NO_VOICE_CONNECTION, NOTHING_IS_PLAYING, PAUSED},
    utils::create_response,
};

pub async fn pause(
    ctx: &Context,
    interaction: &mut ApplicationCommandInteraction,
) -> Result<(), SerenityError> {
    let guild_id = interaction.guild_id.unwrap();
    let manager = songbird::get(ctx).await.unwrap().clone();

    let call = match manager.get(guild_id) {
        Some(call) => call,
        None => return create_response(&ctx.http, interaction, FAIL_NO_VOICE_CONNECTION).await,
    };

    let handler = call.lock().await;
    let queue = handler.queue();

    if queue.is_empty() {
        return create_response(&ctx.http, interaction, NOTHING_IS_PLAYING).await;
    }

    if queue.pause().is_ok() {
        return create_response(&ctx.http, interaction, PAUSED).await;
    }

    Ok(())
}
