use crate::{
    handlers::track_end::update_queue_messages,
    strings::{NOTHING_IS_PLAYING, STOPPED},
    utils::create_response,
};
use serenity::{
    client::Context, model::interactions::application_command::ApplicationCommandInteraction,
};

pub async fn stop(
    ctx: &Context,
    interaction: &mut ApplicationCommandInteraction,
) -> Result<(), Box<dyn std::error::Error>> {
    let guild_id = interaction.guild_id.unwrap();
    let manager = songbird::get(ctx).await.unwrap();
    let call = manager.get(guild_id).unwrap();

    let handler = call.lock().await;
    let queue = handler.queue();

    if queue.is_empty() {
        return create_response(&ctx.http, interaction, NOTHING_IS_PLAYING).await;
    }

    queue.stop();
    drop(handler);

    create_response(&ctx.http, interaction, STOPPED).await?;
    update_queue_messages(&ctx.http, &ctx.data, &call, guild_id).await;
    Ok(())
}
