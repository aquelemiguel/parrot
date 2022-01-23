use crate::{
    settings::GuildSettingsMap,
    strings::{AUTOPAUSE_OFF, AUTOPAUSE_ON},
    utils::create_response,
};
use serenity::{
    client::Context, model::interactions::application_command::ApplicationCommandInteraction,
    prelude::SerenityError,
};

pub async fn autopause(
    ctx: &Context,
    interaction: &mut ApplicationCommandInteraction,
) -> Result<(), SerenityError> {
    let guild_id = interaction.guild_id.unwrap();
    let mut data = ctx.data.write().await;
    let settings = data.get_mut::<GuildSettingsMap>().unwrap();

    let guild_settings = settings.entry(guild_id).or_default();
    guild_settings.autopause = !guild_settings.autopause;

    let message = if guild_settings.autopause {
        AUTOPAUSE_ON.to_string()
    } else {
        AUTOPAUSE_OFF.to_string()
    };
    create_response(&ctx.http, interaction, &message).await
}
