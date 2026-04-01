use crate::{
    errors::ParrotError,
    guild::settings::{GuildSettings, GuildSettingsMap},
    messaging::messages::{
        DOMAIN_FORM_ALLOWED_PLACEHOLDER, DOMAIN_FORM_ALLOWED_TITLE, DOMAIN_FORM_BANNED_PLACEHOLDER,
        DOMAIN_FORM_BANNED_TITLE, DOMAIN_FORM_TITLE,
    },
};
use serenity::{
    all::{
        ActionRowComponent, CommandInteraction, CreateActionRow, CreateInputText,
        CreateInteractionResponse, CreateModal, InputTextStyle,
    },
    client::Context,
    futures::StreamExt,
};

pub async fn allow(ctx: &Context, interaction: &mut CommandInteraction) -> Result<(), ParrotError> {
    let guild_id = interaction.guild_id.ok_or(ParrotError::Other(
        "This command can only be used in a server",
    ))?;

    let mut data = ctx.data.write().await;
    let settings = data
        .get_mut::<GuildSettingsMap>()
        .ok_or(ParrotError::Other("Guild settings not initialized"))?;

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

    let allowed_input = CreateInputText::new(
        InputTextStyle::Paragraph,
        DOMAIN_FORM_ALLOWED_TITLE,
        "allowed_domains",
    )
    .placeholder(DOMAIN_FORM_ALLOWED_PLACEHOLDER)
    .value(allowed_str)
    .required(false);

    let banned_input = CreateInputText::new(
        InputTextStyle::Paragraph,
        DOMAIN_FORM_BANNED_TITLE,
        "banned_domains",
    )
    .placeholder(DOMAIN_FORM_BANNED_PLACEHOLDER)
    .value(banned_str)
    .required(false);

    let modal = CreateModal::new("manage_domains", DOMAIN_FORM_TITLE).components(vec![
        CreateActionRow::InputText(allowed_input),
        CreateActionRow::InputText(banned_input),
    ]);

    let response = CreateInteractionResponse::Modal(modal);
    interaction.create_response(&ctx.http, response).await?;

    // collect the submitted data
    let mut collector = interaction
        .get_response(&ctx.http)
        .await?
        .await_modal_interaction(ctx)
        .stream();

    while let Some(int) = collector.next().await {
        let mut data = ctx.data.write().await;
        let Some(settings) = data.get_mut::<GuildSettingsMap>() else {
            eprintln!("[ERROR] Guild settings not initialized");
            continue;
        };

        let inputs: Vec<_> = int
            .data
            .components
            .iter()
            .flat_map(|r| r.components.iter())
            .collect();

        let Some(guild_settings) = settings.get_mut(&guild_id) else {
            eprintln!("[ERROR] Guild settings not found for {:?}", guild_id);
            continue;
        };

        for input in inputs.iter() {
            if let ActionRowComponent::InputText(it) = input {
                if it.custom_id == "allowed_domains" {
                    if let Some(ref value) = it.value {
                        guild_settings.set_allowed_domains(value);
                    }
                }

                if it.custom_id == "banned_domains" {
                    if let Some(ref value) = it.value {
                        guild_settings.set_banned_domains(value);
                    }
                }
            }
        }

        guild_settings.update_domains();
        if let Err(err) = guild_settings.save() {
            eprintln!("[ERROR] Failed to save guild settings: {}", err);
        }

        // it's now safe to close the modal, so send a response to it
        int.create_response(&ctx.http, CreateInteractionResponse::Acknowledge)
            .await
            .ok();
    }

    Ok(())
}
