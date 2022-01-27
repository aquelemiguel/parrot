use serenity::{
    builder::CreateEmbed,
    http::Http,
    model::{
        channel::Message,
        guild::Guild,
        id::ChannelId,
        interactions::{
            application_command::ApplicationCommandInteraction, InteractionResponseType,
        },
        prelude::User,
    },
    prelude::SerenityError,
};
use songbird::{tracks::TrackHandle, Call};
use std::{sync::Arc, time::Duration};
use tokio::sync::MutexGuard;

use crate::strings::QUEUE_NOW_PLAYING;

pub async fn create_response(
    http: &Arc<Http>,
    interaction: &mut ApplicationCommandInteraction,
    content: &str,
) -> Result<(), SerenityError> {
    let mut embed = CreateEmbed::default();
    embed.description(content);
    create_embed_response(http, interaction, embed).await
}

pub async fn edit_response(
    http: &Arc<Http>,
    interaction: &mut ApplicationCommandInteraction,
    content: &str,
) -> Result<Message, SerenityError> {
    let mut embed = CreateEmbed::default();
    embed.description(content);
    edit_embed_response(http, interaction, embed).await
}

pub async fn create_embed_response(
    http: &Arc<Http>,
    interaction: &mut ApplicationCommandInteraction,
    embed: CreateEmbed,
) -> Result<(), SerenityError> {
    interaction
        .create_interaction_response(&http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| message.add_embed(embed))
        })
        .await
}

pub async fn edit_embed_response(
    http: &Arc<Http>,
    interaction: &mut ApplicationCommandInteraction,
    embed: CreateEmbed,
) -> Result<Message, SerenityError> {
    interaction
        .edit_original_interaction_response(http, |message| message.content(" ").add_embed(embed))
        .await
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

pub fn get_voice_channel_for_user(guild: &Guild, user: &User) -> Option<ChannelId> {
    guild
        .voice_states
        .get(&user.id)
        .and_then(|voice_state| voice_state.channel_id)
}

pub fn is_user_listening_to_bot(guild: &Guild, user: &User, handler: &MutexGuard<Call>) -> bool {
    let bot_channel = match handler.current_channel() {
        Some(channel) => channel,
        None => return false,
    };

    match get_voice_channel_for_user(guild, user) {
        Some(user_channel) => user_channel.0 == bot_channel.0,
        None => false,
    }
}
