use serenity::{
    builder::CreateEmbed,
    http::Http,
    model::{
        application::interaction::{
            application_command::ApplicationCommandInteraction, InteractionResponseType,
        },
        channel::Message,
    },
};
use songbird::tracks::TrackHandle;
use std::{sync::Arc, time::Duration};

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

pub async fn create_followup_text(
    http: &Arc<Http>,
    interaction: &mut ApplicationCommandInteraction,
    content: &str,
) -> Result<Message, ParrotError> {
    let mut embed = CreateEmbed::default();
    embed.description(content);
    create_embed_followup(http, interaction, embed).await
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
    let response: Result<(), ParrotError> = interaction
        .create_interaction_response(&http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| message.add_embed(embed.clone()))
        })
        .await
        .map_err(Into::into);
    match response {
        Ok(val) => Ok(val),
        Err(..) => edit_embed_response(http, interaction, embed)
            .await
            .map(|_| ()),
    }
}

pub async fn create_embed_followup(
    http: &Arc<Http>,
    interaction: &mut ApplicationCommandInteraction,
    embed: CreateEmbed,
) -> Result<Message, ParrotError> {
    interaction
        .create_followup_message(&http, |followup| followup.add_embed(embed))
        .await
        .map_err(Into::into)
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

    embed.field(
        ParrotMessage::NowPlaying,
        format!(
            "[**{}**]({})",
            metadata.title.unwrap(),
            metadata.source_url.unwrap()
        ),
        false,
    );
    embed.thumbnail(&metadata.thumbnail.unwrap());

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
