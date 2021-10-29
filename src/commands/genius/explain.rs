use crate::{
    commands::genius::genius_search, strings::MISSING_PLAY_QUERY, utils::send_simple_message,
};

use serde_json::Value;
use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};

use super::{genius_description, genius_song};

#[command]
async fn explain(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    match args.remains() {
        Some(query) => {
            if let Some(hits) = genius_search(query).await {
                if hits.is_empty() {
                    send_simple_message(&ctx.http, msg, "Could not fetch explanation!").await;
                    return Ok(());
                }

                let id = hits[0]["result"]["id"].as_i64().unwrap();
                let song = genius_song(id).await.unwrap();

                match genius_description(&song).await {
                    Ok(explanation) => {
                        send_explanation_message(ctx, msg, &explanation, &song).await
                    }
                    Err(_) => {
                        send_simple_message(&ctx.http, msg, "Could not fetch explanation!").await
                    }
                }
            } else {
                send_simple_message(
                    &ctx.http,
                    msg,
                    &format!("Could not find any songs that match `{}`", query),
                )
                .await;
            }
        }
        None => {
            send_simple_message(&ctx.http, msg, MISSING_PLAY_QUERY).await;
        }
    };

    Ok(())
}

async fn send_explanation_message(
    ctx: &Context,
    msg: &Message,
    explanation: &String,
    song: &Value,
) {
    msg.channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                let song_title = song["title"].as_str().unwrap().to_string();
                let artist = song["primary_artist"]["name"].as_str().unwrap().to_string();

                e.title(format!(
                    "Explaining {}",
                    format!("\"{}\" by {}", song_title, artist)
                ));

                let url = song["url"].as_str().unwrap().to_string();
                e.url(url);

                let thumbnail = song["song_art_image_url"].as_str().unwrap().to_string();
                e.thumbnail(thumbnail);

                e.description(explanation);

                e.footer(|f| {
                    f.text("Powered by Genius");
                    f.icon_url("https://bit.ly/3BOic6A")
                })
            })
        })
        .await
        .unwrap();
}
