use std::{sync::Arc, time::Duration};

use serenity::{
    http::Http,
    model::interactions::{
        application_command::ApplicationCommandInteraction, InteractionResponseType,
    },
};

use serenity::prelude::SerenityError;
use songbird::tracks::TrackHandle;

// use serde_json::Value;
// use serenity::{
//     framework::standard::CommandResult,
//     http::Http,
//     model::{channel::Message, prelude::User},
//     utils::Color,
// };
// use songbird::tracks::TrackHandle;
// use std::{
//     fs::{self, OpenOptions},
//     io::BufReader,
//     sync::Arc,
//     time::Duration,
// };

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

pub async fn create_queued_response(
    http: &Arc<Http>,
    interaction: &mut ApplicationCommandInteraction,
    title: &str,
    track: &TrackHandle,
    estimated_time: Duration,
) -> Result<(), SerenityError> {
    let metadata = track.metadata().clone();

    interaction
        .create_interaction_response(http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.create_embed(|e| {
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
        })
        .await
}

// pub fn get_full_username(user: &User) -> String {
//     format!("{}#{:04}", user.name, user.discriminator)
// }

pub fn get_human_readable_timestamp(duration: Duration) -> String {
    let seconds = duration.as_secs() % 60;
    let minutes = (duration.as_secs() / 60) % 60;
    let hours = duration.as_secs() / 3600;

    if hours < 1 {
        format!("{:02}:{:02}", minutes, seconds)
    } else {
        format!("{}:{:02}:{:02}", hours, minutes, seconds)
    }
}

// pub fn get_prefixes() -> serde_json::Value {
//     let file_exists = fs::metadata("prefixes.json").is_ok();
//     let file = OpenOptions::new()
//         .read(true)
//         .write(true)
//         .create(true)
//         .open("prefixes.json")
//         .unwrap();

//     if !file_exists {
//         fs::write("prefixes.json", "{}").unwrap();
//     };

//     serde_json::from_reader(BufReader::new(file)).unwrap()
// }

// pub fn merge_json(a: &mut Value, b: &Value) {
//     match (a, b) {
//         (&mut Value::Object(ref mut a), &Value::Object(ref b)) => {
//             for (k, v) in b {
//                 merge_json(a.entry(k.clone()).or_insert(Value::Null), v);
//             }
//         }
//         (a, b) => {
//             *a = b.clone();
//         }
//     }
// }
