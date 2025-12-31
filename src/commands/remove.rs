use crate::{
    commands::play::get_track_metadata,
    errors::{verify, ParrotError},
    handlers::track_end::update_queue_messages,
    messaging::message::ParrotMessage,
    messaging::messages::REMOVED_QUEUE,
    utils::create_embed_response,
    utils::create_response,
};
use serenity::{all::CommandInteraction, builder::CreateEmbed, client::Context};
use songbird::tracks::TrackHandle;
use std::cmp::min;

pub async fn remove(
    ctx: &Context,
    interaction: &mut CommandInteraction,
) -> Result<(), ParrotError> {
    let guild_id = interaction
        .guild_id
        .ok_or(ParrotError::Other("This command can only be used in a server"))?;

    let manager = songbird::get(ctx)
        .await
        .ok_or(ParrotError::Other("Voice manager not configured"))?;

    let call = manager.get(guild_id).ok_or(ParrotError::NotConnected)?;

    let args = interaction.data.options.clone();

    let remove_index = args
        .first()
        .and_then(|opt| opt.value.as_i64())
        .unwrap_or(1) as usize;

    let remove_until = match args.get(1) {
        Some(arg) => arg.value.as_i64().unwrap_or(remove_index as i64) as usize,
        None => remove_index,
    };

    let handler = call.lock().await;
    let queue = handler.queue().current_queue();

    let queue_len = queue.len();
    let remove_until = min(remove_until, queue_len.saturating_sub(1));

    verify(queue_len > 1, ParrotError::QueueEmpty)?;
    verify(
        remove_index < queue_len,
        ParrotError::NotInRange("index", remove_index as isize, 1, queue_len as isize),
    )?;
    verify(
        remove_until >= remove_index,
        ParrotError::NotInRange(
            "until",
            remove_until as isize,
            remove_index as isize,
            queue_len as isize,
        ),
    )?;

    let track = queue.get(remove_index).unwrap();

    handler.queue().modify_queue(|v| {
        v.drain(remove_index..=remove_until);
    });

    // refetch the queue after modification
    let queue = handler.queue().current_queue();
    drop(handler);

    if remove_until == remove_index {
        let embed = create_remove_enqueued_embed(track).await;
        create_embed_response(&ctx.http, interaction, embed).await?;
    } else {
        create_response(&ctx.http, interaction, ParrotMessage::RemoveMultiple).await?;
    }

    update_queue_messages(&ctx.http, &ctx.data, &queue, guild_id).await;
    Ok(())
}

async fn create_remove_enqueued_embed(track: &TrackHandle) -> CreateEmbed {
    let metadata = get_track_metadata(track).unwrap_or_default();

    let mut embed = CreateEmbed::new().field(
        REMOVED_QUEUE,
        format!(
            "[**{}**]({})",
            metadata.title.unwrap_or_default(),
            metadata.source_url.unwrap_or_default()
        ),
        false,
    );

    if let Some(thumbnail) = metadata.thumbnail {
        embed = embed.thumbnail(thumbnail);
    }

    embed
}
