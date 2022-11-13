use crate::{
    errors::ParrotError,
    guild::settings::GuildSettingsMap,
    messaging::messages::{
        DOMAIN_FORM_ALLOWED_TITLE, DOMAIN_FORM_BANNED_TITLE, DOMAIN_FORM_PLACEHOLDER,
        DOMAIN_FORM_TITLE,
    },
};
use serenity::{
    builder::{CreateComponents, CreateInputText},
    client::Context,
    model::{
        application::interaction::application_command::ApplicationCommandInteraction,
        prelude::{component::InputTextStyle, interaction::InteractionResponseType},
    },
};

pub async fn allow(
    ctx: &Context,
    interaction: &mut ApplicationCommandInteraction,
) -> Result<(), ParrotError> {
    let guild_id = interaction.guild_id.unwrap();

    let mut data = ctx.data.write().await;
    let settings = data.get_mut::<GuildSettingsMap>().unwrap();

    // fetch the allowed domains and transform to a comma-separated string
    let guild_settings = settings.entry(guild_id).or_default();

    // transform the domain sets from the settings into a string
    let allowed_str = guild_settings
        .allowed_domains
        .clone()
        .into_iter()
        .collect::<Vec<String>>()
        .join(";");

    let banned_str = guild_settings
        .banned_domains
        .clone()
        .into_iter()
        .collect::<Vec<String>>()
        .join(";");

    let mut allowed_input = CreateInputText::default();

    allowed_input
        .label(DOMAIN_FORM_ALLOWED_TITLE)
        .custom_id("allowed_domains")
        .style(InputTextStyle::Paragraph)
        .placeholder(DOMAIN_FORM_PLACEHOLDER)
        .value(allowed_str)
        .required(false);

    let mut banned_input = CreateInputText::default();

    banned_input
        .label(DOMAIN_FORM_BANNED_TITLE)
        .custom_id("banned_domains")
        .style(InputTextStyle::Paragraph)
        .placeholder(DOMAIN_FORM_PLACEHOLDER)
        .value(banned_str)
        .required(false);

    let mut components = CreateComponents::default();

    components
        .create_action_row(|r| r.add_input_text(allowed_input))
        .create_action_row(|r| r.add_input_text(banned_input));

    interaction
        .create_interaction_response(&ctx.http, |r| {
            r.interaction_response_data(|m| {
                m.title(DOMAIN_FORM_TITLE)
                    .custom_id("manage_domains")
                    .set_components(components)
            })
            .kind(InteractionResponseType::Modal)
        })
        .await
        .unwrap();

    let Ok(message) = interaction.get_interaction_response(&ctx.http).await else {
        return Err(ParrotError::Other("failed to send modal"));
    };

    message
        .await_modal_interaction(&ctx.shard)
        .message_id(message.id)
        .filter(|res| {
            println!("{:?}", res.data.components);
            true
        })
        .await;

    Ok(())
}
