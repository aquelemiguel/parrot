use serenity::{
    builder::CreateEmbed,
    http::{Http, HttpError},
    model::{
        application::interaction::{
            application_command::ApplicationCommandInteraction, InteractionResponseType,
        },
        channel::Message,
    },
    Error,
};
use songbird::tracks::TrackHandle;
use std::{sync::Arc, time::Duration};
use url::Url;

use crate::{errors::ParrotError, messaging::message::ParrotMessage};

pub async fn create_response(
    http: &Arc<Http>,
    interaction: &mut ApplicationCommandInteraction,
    message: ParrotMessage,
) -> Result<(), ParrotError> {
    let mut embed = CreateEmbed::default();
    embed.description(format!("{message}"));
    create_embed_response(http, interaction, embed).await
}

pub async fn create_response_text(
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
    message: ParrotMessage,
) -> Result<Message, ParrotError> {
    let mut embed = CreateEmbed::default();
    embed.description(format!("{message}"));
    edit_embed_response(http, interaction, embed).await
}

pub async fn edit_response_text(
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
    match interaction
        .create_interaction_response(&http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| message.add_embed(embed.clone()))
        })
        .await
        .map_err(Into::into)
    {
        Ok(val) => Ok(val),
        Err(err) => match err {
            ParrotError::Serenity(ref boxed) => match boxed.as_ref() {
                Error::Http(e) => match e.as_ref() {
                    HttpError::UnsuccessfulRequest(req) => match req.error.code {
                        40060 => edit_embed_response(http, interaction, embed)
                            .await
                            .map(|_| ()),
                        _ => Err(err),
                    },
                    _ => Err(err),
                },
                _ => Err(err),
            },
            _ => Err(err),
        },
    }
}

pub async fn edit_embed_response(
    http: &Arc<Http>,
    interaction: &mut ApplicationCommandInteraction,
    embed: CreateEmbed,
) -> Result<Message, ParrotError> {
    interaction
        .edit_original_interaction_response(&http, |message| message.content(" ").add_embed(embed))
        .await
        .map_err(Into::into)
}

pub async fn create_now_playing_embed(track: &TrackHandle) -> CreateEmbed {
    let mut embed = CreateEmbed::default();
    let metadata = track.metadata().clone();

    embed.author(|author| author.name(ParrotMessage::NowPlaying));
    embed.title(metadata.title.unwrap());
    embed.url(metadata.source_url.as_ref().unwrap());

    let position = get_human_readable_timestamp(Some(track.get_info().await.unwrap().position));
    let duration = get_human_readable_timestamp(metadata.duration);

    embed.field("Progress", format!(">>> {} / {}", position, duration), true);

    match metadata.channel {
        Some(channel) => embed.field("Channel", format!(">>> {}", channel), true),
        None => embed.field("Channel", ">>> N/A", true),
    };

    embed.thumbnail(metadata.thumbnail.unwrap());

    let source_url = metadata.source_url.as_ref().unwrap();

    let (footer_text, footer_icon_url) = get_footer_info(source_url);
    embed.footer(|f| f.text(footer_text).icon_url(footer_icon_url));

    embed
}

pub fn get_footer_info(url: &str) -> (String, String) {
    let url_data = Url::parse(url).unwrap();
    let domain = url_data.host_str().unwrap();

    // remove www prefix because it looks ugly
    let domain = domain.replace("www.", "");

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
