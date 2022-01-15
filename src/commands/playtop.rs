use crate::commands::{play::_play, PlayFlag};
use serenity::{
    client::Context, model::interactions::application_command::ApplicationCommandInteraction,
    prelude::SerenityError,
};

async fn playtop(
    ctx: &Context,
    interaction: &mut ApplicationCommandInteraction,
) -> Result<(), SerenityError> {
    _play(ctx, interaction, &PlayFlag::PLAYTOP).await
}
