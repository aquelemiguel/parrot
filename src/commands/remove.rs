use crate::{
    errors::{verify, ParrotError},
    handlers::track_end::update_queue_messages,
    messaging::message::ParrotMessage,
    messaging::messages::REMOVED_QUEUE,
    utils::create_embed_response,
    utils::create_response,
};
use serenity::{
    builder::CreateEmbed, client::Context,
    model::application::interaction::application_command::ApplicationCommandInteraction,
};
use songbird::tracks::TrackHandle;
use std::cmp::min;

pub async fn remove(
    ctx: &Context,
    interaction: &mut ApplicationCommandInteraction,
) -> Result<(), ParrotError> {
    let guild_id = interaction.guild_id.unwrap();
    let manager = songbird::get(ctx).await.unwrap();
    let call = manager.get(guild_id).unwrap();

    let args = interaction.data.options.clone();

    let remove_index = args
        .first()
        .unwrap()
        .value
        .as_ref()
        .unwrap()
        .as_u64()
        .unwrap() as usize;

    let remove_until = match args.get(1) {
        Some(arg) => arg.value.as_ref().unwrap().as_u64().unwrap() as usize,
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
    let mut embed = CreateEmbed::default();
    let metadata = track.metadata().clone();

    embed.field(
        REMOVED_QUEUE,
        format!(
            "[**{}**]({})",
            metadata.title.unwrap(),
            metadata.source_url.unwrap()
        ),
        false,
    );
    embed.thumbnail(metadata.thumbnail.unwrap());

    embed
}
