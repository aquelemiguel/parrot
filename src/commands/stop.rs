use serenity::{
    client::Context, model::interactions::application_command::ApplicationCommandInteraction,
    prelude::SerenityError,
};

use crate::{
    handlers::track_end::update_queue_messages,
    strings::{NO_VOICE_CONNECTION, QUEUE_IS_EMPTY},
    utils::create_response,
};

pub async fn stop(
    ctx: &Context,
    interaction: &mut ApplicationCommandInteraction,
) -> Result<(), SerenityError> {
    let guild_id = interaction.guild_id.unwrap();
    let manager = songbird::get(ctx).await.unwrap();

    let call = match manager.get(guild_id) {
        Some(call) => call,
        None => return create_response(&ctx.http, interaction, NO_VOICE_CONNECTION).await,
    };

    let handler = call.lock().await;
    let queue = handler.queue();

    if queue.is_empty() {
        return create_response(&ctx.http, interaction, QUEUE_IS_EMPTY).await;
    }

    queue.stop();
    drop(handler);

    create_response(&ctx.http, interaction, "Stopped!").await?;
    update_queue_messages(&ctx.http, &ctx.data, &call, guild_id).await;
    Ok(())
}
