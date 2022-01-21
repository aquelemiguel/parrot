use serenity::{
    builder::CreateEmbed, client::Context,
    model::interactions::application_command::ApplicationCommandInteraction,
    prelude::SerenityError,
};

use crate::{
    strings::{NO_SONG_ON_INDEX, NO_VOICE_CONNECTION, QUEUE_IS_EMPTY},
    utils::{create_embed_response, create_response},
};

use songbird::tracks::TrackHandle;

pub async fn remove(
    ctx: &Context,
    interaction: &mut ApplicationCommandInteraction,
) -> Result<(), SerenityError> {
    let guild_id = interaction.guild_id.unwrap();
    let manager = songbird::get(ctx).await.unwrap();

    let call = match manager.get(guild_id) {
        Some(call) => call,
        None => return create_response(&ctx.http, interaction, NO_VOICE_CONNECTION).await,
    };

    let args = interaction.data.options.clone();

    let signed_remove_index = args
        .first()
        .unwrap()
        .value
        .as_ref()
        .unwrap()
        .as_i64()
        .unwrap();

    if !signed_remove_index.is_positive() {
        return create_response(&ctx.http, interaction, "Please provide an index >= 1!").await;
    }

    let remove_index = signed_remove_index as usize;
    let handler = call.lock().await;
    let queue = handler.queue().current_queue();

    if queue.is_empty() {
        create_response(&ctx.http, interaction, QUEUE_IS_EMPTY).await
    } else if queue.len() < remove_index + 1 {
        create_response(&ctx.http, interaction, NO_SONG_ON_INDEX).await
    } else {
        let track = queue.get(remove_index).unwrap();
        handler.queue().modify_queue(|v| {
            v.remove(remove_index);
        });

        let embed = create_remove_enqueued_embed(&track).await;
        create_embed_response(&ctx.http, interaction, embed).await
    }
}

async fn create_remove_enqueued_embed(track: &TrackHandle) -> CreateEmbed {
    let mut embed = CreateEmbed::default();
    let metadata = track.metadata().clone();

    embed.field(
        "❌  Removed from queue",
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
