use crate::{
    errors::ParrotError,
    handlers::track_end::update_queue_messages,
    strings::{REMOVED_QUEUE, REMOVED_QUEUE_MULTIPLE},
    utils::create_embed_response,
    utils::create_response,
};
use serenity::{
    builder::CreateEmbed, client::Context,
    model::interactions::application_command::ApplicationCommandInteraction,
    prelude::SerenityError,
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
    let remove_until = min(remove_until, queue.len() - 1);

    let queue_len = queue.len();
    if queue_len <= 1 {
        return Err(ParrotError::QueueEmpty);
    } else if queue_len < remove_index + 1 {
        return Err(SerenityError::NotInRange(
            "remove_index",
            remove_index as u64,
            1,
            queue_len as u64,
        ))
        .map_err(Into::into);
    } else if remove_until < remove_index {
        return Err(SerenityError::NotInRange(
            "remove_until",
            remove_until as u64,
            remove_index as u64,
            queue_len as u64,
        ))
        .map_err(Into::into);
    }

    let track = queue.get(remove_index).unwrap();

    handler.queue().modify_queue(|v| {
        v.drain(remove_index..=remove_until);
    });
    drop(handler);

    if remove_until == remove_index {
        let embed = create_remove_enqueued_embed(track).await;
        create_embed_response(&ctx.http, interaction, embed).await?;
    } else {
        create_response(&ctx.http, interaction, REMOVED_QUEUE_MULTIPLE).await?;
    }
    update_queue_messages(&ctx.http, &ctx.data, &call, guild_id).await;

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
