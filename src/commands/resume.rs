use crate::{
    errors::{verify, ParrotError},
    messaging::Response,
    utils::create_response_,
};
use serenity::{
    client::Context, model::interactions::application_command::ApplicationCommandInteraction,
};

pub async fn resume(
    ctx: &Context,
    interaction: &mut ApplicationCommandInteraction,
) -> Result<(), ParrotError> {
    let guild_id = interaction.guild_id.unwrap();
    let manager = songbird::get(ctx).await.unwrap();
    let call = manager.get(guild_id).unwrap();

    let handler = call.lock().await;
    let queue = handler.queue();

    verify(!queue.is_empty(), ParrotError::NothingPlaying)?;
    verify(queue.resume(), ParrotError::Other("Failed resuming track"))?;

    create_response_(&ctx.http, interaction, Response::Resume).await
}
