use serenity::{
    client::Context, model::interactions::application_command::ApplicationCommandInteraction,
    prelude::SerenityError,
};

use crate::{
    strings::{CLEARED, FAIL_NO_VOICE_CONNECTION, QUEUE_IS_EMPTY},
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
        None => return create_response(&ctx.http, interaction, FAIL_NO_VOICE_CONNECTION).await,
    };

    let handler = call.lock().await;
    let queue = handler.queue().current_queue();

    if queue.len() <= 1 {
        return create_response(&ctx.http, interaction, QUEUE_IS_EMPTY).await;
    }

    handler.queue().modify_queue(|v| {
        v.drain(1..);
    });

    create_response(&ctx.http, interaction, CLEARED).await
}
