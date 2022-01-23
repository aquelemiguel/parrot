use serenity::{
    client::Context, model::interactions::application_command::ApplicationCommandInteraction,
    prelude::SerenityError,
};

use crate::{
    events::modify_queue_handler::update_queue_messages,
    strings::{NO_VOICE_CONNECTION, QUEUE_IS_EMPTY},
    utils::create_response,
};

pub async fn clear(
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
    let queue = handler.queue().current_queue();

    if queue.is_empty() {
        return create_response(&ctx.http, interaction, QUEUE_IS_EMPTY).await;
    }

    handler.queue().modify_queue(|v| {
        v.drain(1..);
    });

    drop(handler);

    create_response(&ctx.http, interaction, "Cleared!").await?;
    update_queue_messages(&ctx.http, &ctx.data, &call, guild_id).await;
    Ok(())
}
