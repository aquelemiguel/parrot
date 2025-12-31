use serenity::{
    all::{CommandInteraction, CreateInteractionResponse, CreateInteractionResponseMessage, EditInteractionResponse},
    builder::CreateEmbed,
    http::{Http, HttpError},
    model::channel::Message,
    Error,
};
use songbird::tracks::TrackHandle;
use std::{sync::Arc, time::Duration};
use url::Url;

use crate::{
    commands::play::get_track_metadata,
    errors::ParrotError,
    messaging::message::ParrotMessage,
};

pub async fn create_response(
    http: &Arc<Http>,
    interaction: &mut CommandInteraction,
    message: ParrotMessage,
) -> Result<(), ParrotError> {
    let embed = CreateEmbed::new().description(format!("{message}"));
    create_embed_response(http, interaction, embed).await
}

pub async fn create_response_text(
    http: &Arc<Http>,
    interaction: &mut CommandInteraction,
    content: &str,
) -> Result<(), ParrotError> {
    let embed = CreateEmbed::new().description(content);
    create_embed_response(http, interaction, embed).await
}

pub async fn edit_response(
    http: &Arc<Http>,
    interaction: &mut CommandInteraction,
    message: ParrotMessage,
) -> Result<Message, ParrotError> {
    let embed = CreateEmbed::new().description(format!("{message}"));
    edit_embed_response(http, interaction, embed).await
}

pub async fn edit_response_text(
    http: &Arc<Http>,
    interaction: &mut CommandInteraction,
    content: &str,
) -> Result<Message, ParrotError> {
    let embed = CreateEmbed::new().description(content);
    edit_embed_response(http, interaction, embed).await
}

pub async fn create_embed_response(
    http: &Arc<Http>,
    interaction: &mut CommandInteraction,
    embed: CreateEmbed,
) -> Result<(), ParrotError> {
    let response = CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new().add_embed(embed.clone())
    );

    match interaction
        .create_response(&http, response)
        .await
        .map_err(Into::into)
    {
        Ok(val) => Ok(val),
        Err(err) => match &err {
            ParrotError::Serenity(boxed) => match boxed.as_ref() {
                Error::Http(HttpError::UnsuccessfulRequest(req)) => {
                    if req.error.code == 40060 {
                        edit_embed_response(http, interaction, embed)
                            .await
                            .map(|_| ())
                    } else {
                        Err(err)
                    }
                }
                _ => Err(err),
            },
            _ => Err(err),
        },
    }
}

pub async fn edit_embed_response(
    http: &Arc<Http>,
    interaction: &mut CommandInteraction,
    embed: CreateEmbed,
) -> Result<Message, ParrotError> {
    let edit = EditInteractionResponse::new().content(" ").add_embed(embed);
    interaction
        .edit_response(&http, edit)
        .await
        .map_err(Into::into)
}

pub async fn create_now_playing_embed(track: &TrackHandle) -> CreateEmbed {
    use serenity::all::{CreateEmbedAuthor, CreateEmbedFooter};

    let metadata = get_track_metadata(track).unwrap_or_default();

    let position = match track.get_info().await {
        Ok(info) => get_human_readable_timestamp(Some(info.position)),
        Err(_) => "??:??".to_string(),
    };
    let duration = get_human_readable_timestamp(metadata.duration);

    let channel_value = match metadata.channel {
        Some(channel) => format!(">>> {}", channel),
        None => ">>> N/A".to_string(),
    };

    let source_url = metadata.source_url.clone().unwrap_or_default();
    let (footer_text, footer_icon_url) = get_footer_info(&source_url);

    let mut embed = CreateEmbed::new()
        .author(CreateEmbedAuthor::new(format!("{}", ParrotMessage::NowPlaying)))
        .title(metadata.title.unwrap_or_default())
        .url(&source_url)
        .field("Progress", format!(">>> {} / {}", position, duration), true)
        .field("Channel", channel_value, true)
        .footer(CreateEmbedFooter::new(footer_text).icon_url(footer_icon_url));

    if let Some(thumbnail) = metadata.thumbnail {
        embed = embed.thumbnail(thumbnail);
    }

    embed
}

pub fn get_footer_info(url: &str) -> (String, String) {
    let domain = Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.replace("www.", "")))
        .unwrap_or_else(|| "unknown".to_string());

    (
        format!("Streaming via {}", domain),
        format!("https://www.google.com/s2/favicons?domain={}", domain),
    )
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

pub fn compare_domains(domain: &str, subdomain: &str) -> bool {
    subdomain == domain || subdomain.ends_with(domain)
}
