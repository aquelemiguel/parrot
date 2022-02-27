use crate::{strings::LEAVING, utils::create_response};
use serenity::{
    client::Context, model::interactions::application_command::ApplicationCommandInteraction,
};

pub async fn leave(
    ctx: &Context,
    interaction: &mut ApplicationCommandInteraction,
) -> Result<(), Box<dyn std::error::Error>> {
    let guild_id = interaction.guild_id.unwrap();
    let manager = songbird::get(ctx).await.unwrap();
    manager.remove(guild_id).await.unwrap();

    create_response(&ctx.http, interaction, LEAVING).await
}
