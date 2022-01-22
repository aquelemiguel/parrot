use std::{sync::Arc, time::Duration};

use serenity::{
    builder::CreateEmbed,
    http::Http,
    model::{
        interactions::{
            application_command::ApplicationCommandInteraction, InteractionResponseType,
        },
        prelude::User,
    },
    prelude::SerenityError,
};

use songbird::tracks::TrackHandle;

pub async fn create_response(
    http: &Arc<Http>,
    interaction: &mut ApplicationCommandInteraction,
    content: &str,
) -> Result<(), SerenityError> {
    interaction
        .create_interaction_response(&http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| message.content(content))
        })
        .await
}

pub fn get_full_username(user: &User) -> String {
    format!("{}#{:04}", user.name, user.discriminator)
}

pub fn get_human_readable_timestamp(duration: Option<Duration>) -> String {
    if let Some(duration) = duration {
        if duration == Duration::MAX {
            return "∞".to_string();
        }

        let seconds = duration.as_secs() % 60;
        let minutes = (duration.as_secs() / 60) % 60;
        let hours = duration.as_secs() / 3600;

        if hours < 1 {
            format!("{:02}:{:02}", minutes, seconds)
        } else {
            format!("{}:{:02}:{:02}", hours, minutes, seconds)
        }
    } else {
        "∞".to_string()
    }
}

pub async fn create_queued_embed(
    title: &str,
    track: &TrackHandle,
    estimated_time: Duration,
) -> CreateEmbed {
    let mut embed = CreateEmbed::default();
    let metadata = track.metadata().clone();

    embed.title(title);
    embed.thumbnail(metadata.thumbnail.unwrap());

    embed.description(format!(
        "[**{}**]({})",
        metadata.title.unwrap(),
        metadata.source_url.unwrap()
    ));

    let footer_text = format!(
        "Track duration: {}\nEstimated time until play: {}",
        get_human_readable_timestamp(metadata.duration),
        get_human_readable_timestamp(Some(estimated_time))
    );

    embed.footer(|footer| footer.text(footer_text));
    embed
}

pub async fn create_now_playing_embed(track: &TrackHandle) -> CreateEmbed {
    let mut embed = CreateEmbed::default();
    let metadata = track.metadata().clone();

    embed.title("Now playing");
    embed.thumbnail(metadata.thumbnail.unwrap());

    let description_text = format!(
        "[**{}**]({})",
        metadata.title.unwrap(),
        metadata.source_url.unwrap()
    );

    embed.description(description_text);

    let position = get_human_readable_timestamp(Some(track.get_info().await.unwrap().position));
    let duration = get_human_readable_timestamp(metadata.duration);

    let footer_text = format!("{} / {}", position, duration);
    embed.footer(|footer| footer.text(footer_text));
    embed
}
