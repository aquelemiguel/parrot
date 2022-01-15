use std::sync::Arc;

use crate::{
    strings::{NO_VOICE_CONNECTION, QUEUE_IS_EMPTY},
    utils::{create_response, get_human_readable_timestamp},
};
use serenity::{
    builder::CreateEmbedFooter,
    client::Context,
    http::Http,
    model::interactions::{
        application_command::ApplicationCommandInteraction, InteractionResponseType,
    },
    prelude::SerenityError,
};
use songbird::tracks::TrackHandle;

pub async fn now_playing(
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

    let track = match handler.queue().current() {
        Some(track) => track,
        None => return create_response(&ctx.http, interaction, QUEUE_IS_EMPTY).await,
    };

    send_now_playing_message(&ctx.http, interaction, track).await
}

async fn send_now_playing_message(
    http: &Arc<Http>,
    interaction: &mut ApplicationCommandInteraction,
    track: TrackHandle,
) -> Result<(), SerenityError> {
    let position = track.get_info().await.unwrap().position;
    let duration = track.metadata().duration.unwrap();
    let thumbnail = track.metadata().thumbnail.as_ref().unwrap();

    interaction
        .create_interaction_response(http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.create_embed(|e| {
                        e.title("Now playing");
                        e.thumbnail(thumbnail);

                        let title = track.metadata().title.as_ref().unwrap();
                        let url = track.metadata().source_url.as_ref().unwrap();
                        e.description(format!("[**{}**]({})", title, url));

                        let mut footer = CreateEmbedFooter::default();
                        let position_human = get_human_readable_timestamp(position);
                        let duration_human = get_human_readable_timestamp(duration);

                        footer.text(format!("{} / {}", position_human, duration_human));
                        e.set_footer(footer)
                    })
                })
        })
        .await
}
