use crate::{
    strings::{VERSION, VERSION_LATEST},
    utils::create_response,
};
use serenity::{
    client::Context, model::interactions::application_command::ApplicationCommandInteraction,
    prelude::SerenityError,
};

const RELEASES_LINK: &str = "https://github.com/aquelemiguel/parrot/releases";

pub async fn version(
    ctx: &Context,
    interaction: &mut ApplicationCommandInteraction,
) -> Result<(), SerenityError> {
    let current = option_env!("CARGO_PKG_VERSION").unwrap_or_else(|| "Unknown");
    let current = format!(
        "{} [{}]({}/tag/v{})",
        VERSION, current, RELEASES_LINK, current
    );
    let latest = format!("{}({}/latest)", VERSION_LATEST, RELEASES_LINK);
    let content = format!("{}\n{}", current, latest);

    create_response(&ctx.http, interaction, &content).await
}
