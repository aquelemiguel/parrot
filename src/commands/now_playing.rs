use crate::{
    errors::ParrotError,
    messaging::interaction::{create_embed_response, create_now_playing_embed},
};
use serenity::{
    client::Context,
    model::application::interaction::application_command::ApplicationCommandInteraction,
};

pub async fn now_playing(
    ctx: &Context,
    interaction: &mut ApplicationCommandInteraction,
) -> Result<(), ParrotError> {
    let guild_id = interaction.guild_id.unwrap();
    let manager = songbird::get(ctx).await.unwrap();
    let call = manager.get(guild_id).unwrap();

    let handler = call.lock().await;
    let track = handler
        .queue()
        .current()
        .ok_or(ParrotError::NothingPlaying)?;

    let embed = create_now_playing_embed(&track).await;
    create_embed_response(&ctx.http, interaction, embed).await
}
