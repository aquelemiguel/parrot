use serenity::{
    http::Http,
    model::{channel::Message, prelude::User},
    utils::Color,
};
use songbird::tracks::TrackHandle;
use std::{sync::Arc, time::Duration};

pub async fn send_simple_message(http: &Arc<Http>, msg: &Message, content: &str) {
    msg.channel_id
        .send_message(http, |m| {
            m.embed(|e| e.description(format!("**{}**", content)).color(Color::RED))
        })
        .await
        .expect("Unable to send message");
}

pub async fn send_added_to_queue_message(
    http: &Arc<Http>,
    msg: &Message,
    title: &str,
    track: &TrackHandle,
    estimated_time: Duration,
) {
    let metadata = track.metadata().clone();

    msg.channel_id
        .send_message(http, |m| {
            m.embed(|e| {
                e.title(title);
                e.thumbnail(metadata.thumbnail.unwrap());

                e.description(format!(
                    "[**{}**]({})",
                    metadata.title.unwrap(),
                    metadata.source_url.unwrap()
                ));

                let footer_text = format!(
                    "Track duration: {}\nEstimated time until play: {}",
                    get_human_readable_timestamp(metadata.duration.unwrap()),
                    get_human_readable_timestamp(estimated_time)
                );

                e.footer(|f| f.text(footer_text))
            })
        })
        .await
        .expect("Unable to send message");
}

pub fn get_human_readable_timestamp(duration: Duration) -> String {
    let seconds = duration.as_secs() % 60;
    let minutes = (duration.as_secs() / 60) % 60;
    let hours = duration.as_secs() / 3600;

    if hours < 1 {
        format!("{}:{:02}", minutes, seconds)
    } else {
        format!("{}:{:02}:{:02}", hours, minutes, seconds)
    }
}

pub fn get_full_username(user: &User) -> String {
    format!("{}#{:04}", user.name, user.discriminator)
}
