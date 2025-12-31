use crate::{errors::ParrotError, messaging::message::ParrotMessage, utils::create_response};
use serenity::{all::CommandInteraction, client::Context};

pub async fn version(
    ctx: &Context,
    interaction: &mut CommandInteraction,
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
