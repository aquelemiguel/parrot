use html2md::parse_html;
use reqwest::header::{AUTHORIZATION, COOKIE};
use reqwest::Error;
use scraper::{Html, Selector};
use serde_json::Value;
use std::env;

pub mod explain;
pub mod lyrics;

const GENIUS_BASE_ENDPOINT: &str = "https://api.genius.com/";

pub struct GeniusExplanation {
    text: String,
    thumbnail: String,
    page_url: String,
    song: String,
}

pub async fn genius_search(query: &str) -> Option<Vec<Value>> {
    let res = send_genius_request(format!("search?q={}", query))
        .await
        .unwrap();

    res["response"]["hits"].as_array().map(|v| v.to_owned())
}

pub async fn genius_description(id: i64) -> Result<GeniusExplanation, Error> {
    let res = send_genius_request(format!("songs/{}?text_format=html", id)).await?;
    let song = &res["response"]["song"];

    let song_title = song["title"].as_str().unwrap().to_string();
    let artist = song["primary_artist"]["name"].as_str().unwrap().to_string();
    let thumbnail = song["song_art_image_url"].as_str().unwrap().to_string();
    let page_url = song["url"].as_str().unwrap().to_string();
    let text = parse_html(song["description"]["html"].as_str().unwrap());

    Ok(GeniusExplanation {
        text,
        thumbnail,
        page_url,
        song: format!("\"{}\" by {}", song_title, artist),
    })
}

pub async fn genius_lyrics(url: &str) -> Result<String, Error> {
    let client = reqwest::Client::new();
    let res = client
        .get(url)
        .header(COOKIE, "_genius_ab_test_cohort=33") // Why this?
        .send()
        .await?;

    let document = res.text().await?;
    let fragment = Html::parse_document(&document);
    let selector = Selector::parse(".Lyrics__Container-sc-1ynbvzw-10").unwrap();

    let lyrics = fragment
        .select(&selector)
        .map(|elem| parse_html(&elem.html()))
        .collect::<Vec<String>>()
        .join("");

    Ok(lyrics)
}

pub async fn send_genius_request(resource: String) -> Result<Value, reqwest::Error> {
    let client = reqwest::Client::new();
    let endpoint = format!("{}{}", GENIUS_BASE_ENDPOINT, resource);
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
