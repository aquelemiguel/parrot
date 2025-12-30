use crate::{
    errors::ParrotError,
    guild::settings::{GuildSettings, GuildSettingsMap},
    messaging::messages::{
        DOMAIN_FORM_ALLOWED_PLACEHOLDER, DOMAIN_FORM_ALLOWED_TITLE, DOMAIN_FORM_BANNED_PLACEHOLDER,
        DOMAIN_FORM_BANNED_TITLE, DOMAIN_FORM_TITLE,
    },
};
use serenity::{
    builder::{CreateComponents, CreateInputText},
    client::Context,
    collector::ModalInteractionCollectorBuilder,
    futures::StreamExt,
    model::{
        application::interaction::application_command::ApplicationCommandInteraction,
        prelude::{
            component::{ActionRowComponent, InputTextStyle},
            interaction::InteractionResponseType,
        },
    },
};

pub async fn allow(
    ctx: &Context,
    interaction: &mut ApplicationCommandInteraction,
) -> Result<(), ParrotError> {
    let guild_id = interaction.guild_id.unwrap();

    let mut data = ctx.data.write().await;
    let settings = data.get_mut::<GuildSettingsMap>().unwrap();

    let guild_settings = settings
        .entry(guild_id)
        .or_insert_with(|| GuildSettings::new(guild_id));

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

    drop(data);

    let mut allowed_input = CreateInputText::default();
    allowed_input
        .label(DOMAIN_FORM_ALLOWED_TITLE)
        .custom_id("allowed_domains")
        .style(InputTextStyle::Paragraph)
        .placeholder(DOMAIN_FORM_ALLOWED_PLACEHOLDER)
        .value(allowed_str)
        .required(false);

    let mut banned_input = CreateInputText::default();
    banned_input
        .label(DOMAIN_FORM_BANNED_TITLE)
        .custom_id("banned_domains")
        .style(InputTextStyle::Paragraph)
        .placeholder(DOMAIN_FORM_BANNED_PLACEHOLDER)
        .value(banned_str)
        .required(false);

    let mut components = CreateComponents::default();
    components
        .create_action_row(|r| r.add_input_text(allowed_input))
        .create_action_row(|r| r.add_input_text(banned_input));

    interaction
        .create_interaction_response(&ctx.http, |r| {
            r.kind(InteractionResponseType::Modal);
            r.interaction_response_data(|d| {
                d.title(DOMAIN_FORM_TITLE);
                d.custom_id("manage_domains");
                d.set_components(components)
            })
        })
        .await?;

    // collect the submitted data
    let collector = ModalInteractionCollectorBuilder::new(ctx)
        .filter(|int| int.data.custom_id == "manage_domains")
        .build();

    collector
        .then(|int| async move {
            let mut data = ctx.data.write().await;
            let settings = data.get_mut::<GuildSettingsMap>().unwrap();

            let inputs: Vec<_> = int
                .data
                .components
                .iter()
                .flat_map(|r| r.components.iter())
                .collect();

            let guild_settings = settings.get_mut(&guild_id).unwrap();

            for input in inputs.iter() {
                if let ActionRowComponent::InputText(it) = input {
                    if it.custom_id == "allowed_domains" {
                        guild_settings.set_allowed_domains(&it.value);
                    }

                    if it.custom_id == "banned_domains" {
                        guild_settings.set_banned_domains(&it.value);
                    }
                }
            }

            guild_settings.update_domains();
            if let Err(err) = guild_settings.save() {
                eprintln!("[ERROR] Failed to save guild settings: {}", err);
            }

            // it's now safe to close the modal, so send a response to it
            int.create_interaction_response(&ctx.http, |r| {
                r.kind(InteractionResponseType::DeferredUpdateMessage)
            })
            .await
            .ok();
        })
        .collect::<Vec<_>>()
        .await;

    Ok(())
}
