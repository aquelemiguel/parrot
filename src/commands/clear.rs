use crate::{
    errors::{verify, ParrotError},
    handlers::track_end::update_queue_messages,
    messaging::{interaction::create_response, message::ParrotMessage},
};
use serenity::{
    client::Context,
    model::application::interaction::application_command::ApplicationCommandInteraction,
};

pub async fn clear(
    ctx: &Context,
    interaction: &mut ApplicationCommandInteraction,
) -> Result<(), ParrotError> {
    let guild_id = interaction.guild_id.unwrap();
    let manager = songbird::get(ctx).await.unwrap();
    let call = manager.get(guild_id).unwrap();

    let handler = call.lock().await;
    let queue = handler.queue().current_queue();

    verify(queue.len() > 1, ParrotError::QueueEmpty)?;

    handler.queue().modify_queue(|v| {
        v.drain(1..);
    });

    // refetch the queue after modification
    let queue = handler.queue().current_queue();
    drop(handler);

    create_response(&ctx.http, interaction, ParrotMessage::Clear).await?;
    update_queue_messages(&ctx.http, &ctx.data, &queue, guild_id).await;
    Ok(())
}
