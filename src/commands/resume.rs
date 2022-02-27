use crate::{errors::ParrotError, strings::RESUMED, utils::create_response};
use serenity::{
    client::Context, model::interactions::application_command::ApplicationCommandInteraction,
};

pub async fn resume(
    ctx: &Context,
    interaction: &mut ApplicationCommandInteraction,
) -> Result<(), ParrotError> {
    let guild_id = interaction.guild_id.unwrap();
    let manager = songbird::get(ctx).await.unwrap();
    let call = manager.get(guild_id).unwrap();

    let handler = call.lock().await;
    let queue = handler.queue();

    if queue.is_empty() {
        return Err(ParrotError::NothingPlaying);
    }

    if queue.resume().is_ok() {
        create_response(&ctx.http, interaction, RESUMED).await
    } else {
        Err(ParrotError::Other("Failed resuming current track"))
    }
}
