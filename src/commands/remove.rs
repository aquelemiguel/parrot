use serenity::{
    client::Context, model::interactions::application_command::ApplicationCommandInteraction,
    prelude::SerenityError,
};

use crate::{
    strings::{NO_SONG_ON_INDEX, NO_VOICE_CONNECTION, QUEUE_IS_EMPTY},
    utils::create_response,
};

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

    let remove_index = args
        .first()
        .unwrap()
        .value
        .as_ref()
        .unwrap()
        .as_u64()
        .unwrap() as usize;

    let handler = call.lock().await;
    let queue = handler.queue().current_queue();

    if queue.is_empty() {
        create_response(&ctx.http, interaction, QUEUE_IS_EMPTY).await
    } else if queue.len() < remove_index + 1 {
        create_response(&ctx.http, interaction, NO_SONG_ON_INDEX).await
    } else if remove_index == 0 {
        create_response(
            &ctx.http,
            interaction,
            "Can't remove currently playing song!",
        )
        .await
    } else {
        handler.queue().modify_queue(|v| {
            v.remove(remove_index);
        });
        create_response(
            &ctx.http,
            interaction,
            &format!("Removed track #{}!", remove_index),
        )
        .await
    }
}
