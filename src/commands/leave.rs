use serenity::{
    client::Context, model::interactions::application_command::ApplicationCommandInteraction,
    prelude::SerenityError,
};

use crate::{
    strings::{FAIL_NO_VOICE_CONNECTION, LEAVING},
    utils::create_response,
};

pub async fn leave(
    ctx: &Context,
    interaction: &mut ApplicationCommandInteraction,
) -> Result<(), SerenityError> {
    let guild_id = interaction.guild_id.unwrap();
    let manager = songbird::get(ctx).await.unwrap();

    if manager.get(guild_id).is_some() {
        manager.remove(guild_id).await.unwrap();
        create_response(&ctx.http, interaction, LEAVING).await
    } else {
        create_response(&ctx.http, interaction, FAIL_NO_VOICE_CONNECTION).await
    }
}
