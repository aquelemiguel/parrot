use serenity::{
    client::Context, model::interactions::application_command::ApplicationCommandInteraction,
    prelude::SerenityError,
};

use crate::{strings::NO_VOICE_CONNECTION, utils::create_response};

pub async fn leave(
    ctx: &Context,
    interaction: &mut ApplicationCommandInteraction,
) -> Result<(), SerenityError> {
    let guild_id = interaction.guild_id.unwrap();
    let manager = songbird::get(ctx).await.unwrap();

    if manager.get(guild_id).is_some() {
        manager.remove(guild_id).await.unwrap();
        create_response(&ctx.http, interaction, "See you soon!").await
    } else {
        create_response(&ctx.http, interaction, NO_VOICE_CONNECTION).await
    }
}
