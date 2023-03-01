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
use std::sync::Arc;

use crate::{
    errors::ParrotError,
    messaging::message::ParrotMessage,
    utils::{get_footer_info, get_human_readable_timestamp},
};

pub async fn create_response(
    http: &Arc<Http>,
    interaction: &mut ApplicationCommandInteraction,
    message: ParrotMessage,
) -> Result<(), ParrotError> {
    let mut embed = CreateEmbed::default();
    embed.description(message.localize("en_us"));
    create_embed_response(http, interaction, embed).await
}

pub async fn edit_response(
    http: &Arc<Http>,
    interaction: &mut ApplicationCommandInteraction,
    message: ParrotMessage,
) -> Result<Message, ParrotError> {
    let mut embed = CreateEmbed::default();
    embed.description(message.localize("en_us"));
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
            ParrotError::Serenity(Error::Http(ref e)) => match &**e {
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
    let message = ParrotMessage::NowPlaying;

    embed.author(|author| author.name(message.localize("en_us")));
    embed.title(metadata.title.unwrap());
    embed.url(metadata.source_url.as_ref().unwrap());

    let position = get_human_readable_timestamp(Some(track.get_info().await.unwrap().position));
    let duration = get_human_readable_timestamp(metadata.duration);

    embed.field("Progress", format!(">>> {} / {}", position, duration), true);

    match metadata.channel {
        Some(channel) => embed.field("Channel", format!(">>> {}", channel), true),
        None => embed.field("Channel", ">>> N/A", true),
    };

    embed.thumbnail(&metadata.thumbnail.unwrap());

    let source_url = metadata.source_url.as_ref().unwrap();

    let (footer_text, footer_icon_url) = get_footer_info(source_url);
    embed.footer(|f| f.text(footer_text).icon_url(footer_icon_url));

    embed
}
