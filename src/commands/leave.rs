use crate::{
    errors::ParrotError,
    messaging::{interaction::create_response, message::ParrotMessage},
};
use serenity::{
    client::Context,
    model::application::interaction::application_command::ApplicationCommandInteraction,
};

pub async fn leave(
    ctx: &Context,
    interaction: &mut ApplicationCommandInteraction,
) -> Result<(), ParrotError> {
    let guild_id = interaction.guild_id.unwrap();
    let manager = songbird::get(ctx).await.unwrap();
    manager.remove(guild_id).await.unwrap();

    create_response(&ctx.http, interaction, ParrotMessage::Leaving).await
}
