use crate::{errors::ParrotError, messaging::message::ParrotMessage, utils::create_response};
use serenity::{all::CommandInteraction, client::Context};

pub async fn leave(ctx: &Context, interaction: &mut CommandInteraction) -> Result<(), ParrotError> {
    let guild_id = interaction.guild_id.ok_or(ParrotError::Other(
        "This command can only be used in a server",
    ))?;
    let manager = songbird::get(ctx)
        .await
        .ok_or(ParrotError::Other("Voice manager not configured"))?;
    manager
        .remove(guild_id)
        .await
        .map_err(|_| ParrotError::Other("Failed to leave voice channel"))?;

    create_response(&ctx.http, interaction, ParrotMessage::Leaving).await
}
