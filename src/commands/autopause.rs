use crate::{
    errors::ParrotError,
    guild::settings::{GuildSettings, GuildSettingsMap},
    messaging::message::ParrotMessage,
    utils::create_response,
};
use serenity::{all::CommandInteraction, client::Context};

pub async fn autopause(
    ctx: &Context,
    interaction: &mut CommandInteraction,
) -> Result<(), ParrotError> {
    let guild_id = interaction.guild_id.unwrap();
    let mut data = ctx.data.write().await;
    let settings = data.get_mut::<GuildSettingsMap>().unwrap();

    let guild_settings = settings
        .entry(guild_id)
        .or_insert_with(|| GuildSettings::new(guild_id));
    guild_settings.toggle_autopause();
    guild_settings.save()?;

    if guild_settings.autopause {
        create_response(&ctx.http, interaction, ParrotMessage::AutopauseOn).await
    } else {
        create_response(&ctx.http, interaction, ParrotMessage::AutopauseOff).await
    }
}
