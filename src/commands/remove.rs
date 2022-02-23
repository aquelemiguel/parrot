use crate::{
    handlers::track_end::update_queue_messages,
    strings::{FAIL_NO_SONG_ON_INDEX, QUEUE_IS_EMPTY, REMOVED_QUEUE, REMOVED_QUEUE_MULTIPLE},
    utils::create_embed_response,
    utils::create_response,
};
use serenity::{
    builder::CreateEmbed, client::Context,
    model::interactions::application_command::ApplicationCommandInteraction,
    prelude::SerenityError,
};
use songbird::tracks::TrackHandle;

pub async fn remove(
    ctx: &Context,
    interaction: &mut ApplicationCommandInteraction,
) -> Result<(), SerenityError> {
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

    if queue.len() <= 1 {
        create_response(&ctx.http, interaction, QUEUE_IS_EMPTY).await
    } else if queue.len() < remove_index + 1 {
        create_response(&ctx.http, interaction, FAIL_NO_SONG_ON_INDEX).await
    } else {
        let track = queue.get(remove_index).unwrap();

        handler.queue().modify_queue(|v| {
            v.drain(remove_index..remove_until + 1);
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
