use serenity::{
    client::Context, model::interactions::application_command::ApplicationCommandInteraction,
};

use crate::utils::create_response;
use serenity::prelude::SerenityError;

const RELEASES_LINK: &str = "https://github.com/aquelemiguel/parrot/releases";

pub async fn version(
    ctx: &Context,
    interaction: &mut ApplicationCommandInteraction,
) -> Result<(), SerenityError> {
    let current = option_env!("CARGO_PKG_VERSION").unwrap_or_else(|| "Unknown");
    let current = format!("Version [{}]({}/tag/v{})", current, RELEASES_LINK, current);
    let latest = format!("Find the latest version [here]({}/latest)", RELEASES_LINK);
    let content = format!("{}\n{}", current, latest);

    create_response(&ctx.http, interaction, &content).await
}
