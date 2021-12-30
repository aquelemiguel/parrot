use crate::{
    commands::genius::{genius_lyrics, genius_search, genius_song},
    strings::MISSING_QUERY,
    utils::send_simple_message,
};

use serde_json::Value;
use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};

#[command]
async fn lyrics(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let query = match args.remains() {
        Some(query) => query,
        None => return send_simple_message(&ctx.http, msg, MISSING_QUERY).await,
    };

    let hits = match genius_search(query).await {
        Some(hits) if !hits.is_empty() => hits,
        _ => {
            return send_simple_message(
                &ctx.http,
                msg,
                &format!("Could not find any songs that match `{}`", query),
            )
            .await
        }
    };

    let id = hits[0]["result"]["id"].as_i64().unwrap();
    let song = genius_song(id).await.unwrap();
    let url = hits[0]["result"]["url"].as_str().unwrap();
    match genius_lyrics(url).await {
        Ok(lyrics) => {
            let message = flatten_lyrics(&lyrics);
            send_lyrics_message(ctx, msg, &message, &song).await
        }
        Err(_) => send_simple_message(&ctx.http, msg, "Could not fetch lyrics!").await,
    }
}

fn flatten_lyrics(lyrics: &[String]) -> String {
    lyrics
        .iter()
        .map(|line| {
            // Bolden sections between brackets (e.g. [Verse 1]).
            if line.starts_with('[') && line.ends_with(']') {
                format!("\n**{}**", line)
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<String>>()
        .join("\n")
}

async fn send_lyrics_message(
    ctx: &Context,
    msg: &Message,
    lyrics: &String,
    song: &Value,
) -> CommandResult {
    let mut final_lyrics = lyrics.clone();

    if lyrics.len() > 2048 {
        final_lyrics = format!("{} [...]", &lyrics[..2048]);
    }

    msg.channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                let song_title = song["title"].as_str().unwrap().to_string();
                let artist = song["primary_artist"]["name"].as_str().unwrap().to_string();

                e.title(format!(
                    "Lyrics for {}",
                    format!("\"{}\" by {}", song_title, artist)
                ));

                let url = song["url"].as_str().unwrap().to_string();
                e.url(url);

                let thumbnail = song["song_art_image_url"].as_str().unwrap().to_string();
                e.thumbnail(thumbnail);

                e.description(final_lyrics);

                e.footer(|f| {
                    f.text("Powered by Genius");
                    f.icon_url("https://bit.ly/3BOic6A")
                })
            })
        })
        .await?;
    Ok(())
}
