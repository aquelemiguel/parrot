use crate::{errors::ParrotError, messaging::message::ParrotMessage, utils::create_response};
use serenity::{all::CommandInteraction, client::Context};

pub async fn leave(ctx: &Context, interaction: &mut CommandInteraction) -> Result<(), ParrotError> {
    let guild_id = interaction.guild_id.unwrap();
    let manager = songbird::get(ctx).await.unwrap();
    manager.remove(guild_id).await.unwrap();

    create_response(&ctx.http, interaction, ParrotMessage::Leaving).await
}
