use crate::{
    errors::ParrotError,
    guild::settings::GuildSettingsMap,
    messaging::{interaction::create_response, message::ParrotMessage},
};
use serenity::{
    client::Context,
    model::application::interaction::application_command::ApplicationCommandInteraction,
};

const DEFAULT_LOCALE: &str = "en_us";

pub async fn language(
    ctx: &Context,
    interaction: &mut ApplicationCommandInteraction,
) -> Result<(), ParrotError> {
    let guild_id = interaction.guild_id.unwrap();
    let mut data = ctx.data.write().await;

    let args = interaction.data.options.clone();
    let locale = match args.first() {
        Some(arg) => arg.value.as_ref().unwrap().as_str().unwrap(),
        None => DEFAULT_LOCALE,
    };

    let settings = data.get_mut::<GuildSettingsMap>().unwrap();
    let guild_settings = settings.entry(guild_id).or_default();
    guild_settings.locale = locale.to_owned();

    create_response(&ctx.http, interaction, ParrotMessage::Clear).await
}
