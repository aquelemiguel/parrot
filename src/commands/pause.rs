use crate::{
    errors::{verify, ParrotError},
    messaging::message::ParrotMessage,
    utils::create_response,
};
use serenity::{all::CommandInteraction, client::Context};

pub async fn pause(ctx: &Context, interaction: &mut CommandInteraction) -> Result<(), ParrotError> {
    let guild_id = interaction.guild_id.ok_or(ParrotError::Other(
        "This command can only be used in a server",
    ))?;
    let manager = songbird::get(ctx)
        .await
        .ok_or(ParrotError::Other("Voice manager not configured"))?;
    let call = manager.get(guild_id).ok_or(ParrotError::NotConnected)?;

    let handler = call.lock().await;
    let queue = handler.queue();

    verify(!queue.is_empty(), ParrotError::NothingPlaying)?;
    verify(queue.pause(), ParrotError::Other("Failed to pause"))?;

    create_response(&ctx.http, interaction, ParrotMessage::Pause).await
}
