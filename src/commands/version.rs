use serenity::{
    builder::CreateInteractionResponse,
    client::Context,
    model::interactions::{
        application_command::ApplicationCommandInteraction, InteractionResponseType,
    },
};

const RELEASES_LINK: &str = "https://github.com/aquelemiguel/parrot/releases";

pub async fn version(ctx: &Context, interaction: &mut ApplicationCommandInteraction) {
    let current = option_env!("CARGO_PKG_VERSION").unwrap_or_else(|| "Unknown");
    let current = format!("Version [{}]({}/tag/v{})", current, RELEASES_LINK, current);
    let latest = format!("Find the latest version [here]({}/latest)", RELEASES_LINK);
    let content = format!("{}\n{}", current, latest);

    interaction
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| message.content(content))
        })
        .await
        .unwrap();
}
