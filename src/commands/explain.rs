use std::env;

use crate::{
    strings::MISSING_PLAY_QUERY,
    utils::{get_full_username, send_simple_message},
};
use reqwest::header::AUTHORIZATION;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};

const GENIUS_ENDPOINT_BASE: &str = "https://api.genius.com/";

#[derive(Serialize, Deserialize)]
struct GeniusTag {
    #[serde(default)]
    tag: String,
    #[serde(default)]
    children: Vec<GeniusChild>,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum GeniusChild {
    PlainText(String),
    Child(GeniusTag),
}

struct GeniusExplanation {
    text: String,
    thumbnail: String,
    song: String,
}

#[command]
async fn explain(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let query = match args.remains() {
        Some(q) => q,
        None => {
            send_simple_message(&ctx.http, msg, MISSING_PLAY_QUERY).await;
            return Ok(());
        }
    };

    let song_id = get_genius_song_id(query).await.unwrap();
    let explanation = get_genius_song_explanation(song_id).await;

    msg.channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title(format!("Explaining {}", explanation.song));
                e.thumbnail(explanation.thumbnail);
                e.description(explanation.text);

                let author_username = get_full_username(&msg.author);
                e.footer(|f| {
                    f.text(format!(
                        "Requested by {} • Powered by Genius",
                        author_username
                    ))
                })
            })
        })
        .await?;

    Ok(())
}

async fn get_genius_song_id(query: &str) -> Option<i64> {
    let res = send_genius_request(format!("search?q={}", query))
        .await
        .unwrap();

    let hits = res["response"]["hits"].as_array();

    match hits {
        Some(hits) => {
            if !hits.is_empty() {
                return hits[0]["result"]["id"].as_i64();
            }
            None
        }
        None => None,
    }
}

async fn get_genius_song_explanation(id: i64) -> GeniusExplanation {
    let res = send_genius_request(format!("songs/{}", id)).await.unwrap();

    let song_title = res["response"]["song"]["title"]
        .as_str()
        .unwrap()
        .to_string();

    let artist = res["response"]["song"]["primary_artist"]["name"]
        .as_str()
        .unwrap()
        .to_string();

    let thumbnail = res["response"]["song"]["song_art_image_url"]
        .as_str()
        .unwrap()
        .to_string();

    let dom = res["response"]["song"]["description"]["dom"].clone();
    let dom: GeniusTag = serde_json::from_value(dom).unwrap();

    let mut text = String::new();
    depth_first_search(&dom, &mut text);

    GeniusExplanation {
        text,
        thumbnail,
        song: format!("\"{}\" by {}", song_title, artist),
    }
}

fn depth_first_search(tree: &GeniusTag, desc: &mut String) {
    for child in tree.children.iter() {
        match child {
            GeniusChild::PlainText(text) => {
                if text.is_empty() {
                    desc.push_str("\n\n");
                } else {
                    desc.push_str(text);
                }
            }
            GeniusChild::Child(child) => depth_first_search(child, desc),
        }
    }
}

async fn send_genius_request(resource: String) -> Result<Value, reqwest::Error> {
    let client = reqwest::Client::new();
    let endpoint = format!("{}{}", GENIUS_ENDPOINT_BASE, resource);
    let auth_header = format!("Bearer {}", env::var("GENIUS_TOKEN").unwrap());

    client
        .get(endpoint)
        .header(AUTHORIZATION, auth_header)
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
}
