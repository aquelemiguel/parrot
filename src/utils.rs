use serenity::{
    builder::CreateEmbed,
    http::Http,
    model::{
        channel::Message,
        interactions::{
            application_command::ApplicationCommandInteraction, InteractionResponseType,
        },
    },
};
use songbird::tracks::TrackHandle;
use std::{sync::Arc, time::Duration};

use crate::{errors::ParrotError, messaging::Response, strings::QUEUE_NOW_PLAYING};

pub async fn create_response(
    http: &Arc<Http>,
    interaction: &mut ApplicationCommandInteraction,
    response_type: Response,
) -> Result<(), ParrotError> {
    let mut embed = CreateEmbed::default();
    embed.description(format!("{response_type}"));
    create_embed_response(http, interaction, embed).await
}

#[deprecated(since = "1.4.3", note = "please use `create_response` instead")]
pub async fn create_response_free(
    http: &Arc<Http>,
    interaction: &mut ApplicationCommandInteraction,
    content: &str,
) -> Result<(), ParrotError> {
    let mut embed = CreateEmbed::default();
    embed.description(content);
    create_embed_response(http, interaction, embed).await
}

pub async fn edit_response(
    http: &Arc<Http>,
    interaction: &mut ApplicationCommandInteraction,
    content: &str,
) -> Result<Message, ParrotError> {
    let mut embed = CreateEmbed::default();
    embed.description(content);
    edit_embed_response(http, interaction, embed).await
}

pub async fn create_embed_response(
    http: &Arc<Http>,
    interaction: &mut ApplicationCommandInteraction,
    embed: CreateEmbed,
) -> Result<(), ParrotError> {
    interaction
        .create_interaction_response(&http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| message.add_embed(embed))
        })
        .await
        .map_err(Into::into)
}

pub async fn edit_embed_response(
    http: &Arc<Http>,
    interaction: &mut ApplicationCommandInteraction,
    embed: CreateEmbed,
) -> Result<Message, ParrotError> {
    interaction
        .edit_original_interaction_response(http, |message| message.content(" ").add_embed(embed))
        .await
        .map_err(Into::into)
}

pub async fn create_now_playing_embed(track: &TrackHandle) -> CreateEmbed {
    let mut embed = CreateEmbed::default();
    let metadata = track.metadata().clone();

    embed.field(
        QUEUE_NOW_PLAYING,
        format!(
            "[**{}**]({})",
            metadata.title.unwrap(),
            metadata.source_url.unwrap()
        ),
        false,
    );
    embed.thumbnail(metadata.thumbnail.unwrap());

    let position = get_human_readable_timestamp(Some(track.get_info().await.unwrap().position));
    let duration = get_human_readable_timestamp(metadata.duration);

    let footer_text = format!("{} / {}", position, duration);
    embed.footer(|footer| footer.text(footer_text));
    embed
}

pub fn get_human_readable_timestamp(duration: Option<Duration>) -> String {
    match duration {
        Some(duration) if duration == Duration::MAX => "∞".to_string(),
        Some(duration) => {
            let seconds = duration.as_secs() % 60;
            let minutes = (duration.as_secs() / 60) % 60;
            let hours = duration.as_secs() / 3600;

            if hours < 1 {
                format!("{:02}:{:02}", minutes, seconds)
            } else {
                format!("{}:{:02}:{:02}", hours, minutes, seconds)
            }
        }
        None => "∞".to_string(),
    }
}
