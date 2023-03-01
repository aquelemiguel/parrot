use crate::{
    errors::ParrotError,
    messaging::{interaction::create_response, message::ParrotMessage},
};
use serenity::{
    client::Context,
    model::application::interaction::application_command::ApplicationCommandInteraction,
};

pub async fn version(
    ctx: &Context,
    interaction: &mut ApplicationCommandInteraction,
) -> Result<(), ParrotError> {
    let current = option_env!("CARGO_PKG_VERSION").unwrap_or_else(|| "Unknown");
    create_response(
        &ctx.http,
        interaction,
        ParrotMessage::Version {
            current: current.to_owned(),
        },
    )
    .await
}
