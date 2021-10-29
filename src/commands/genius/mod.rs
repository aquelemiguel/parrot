use html2md::parse_html;
use regex::Regex;
use reqwest::header::{AUTHORIZATION, COOKIE};
use reqwest::Error;
use scraper::{Html, Selector};
use serde_json::Value;
use std::env;

pub mod explain;
pub mod lyrics;

const GENIUS_BASE_ENDPOINT: &str = "https://api.genius.com/";
const LYRICS_SELECTOR: &str = ".Lyrics__Container-sc-1ynbvzw-10";

pub async fn genius_search(query: &str) -> Option<Vec<Value>> {
    let res = send_genius_request(format!("search?q={}", query))
        .await
        .unwrap();

    res["response"]["hits"].as_array().map(|v| v.to_owned())
}

pub async fn genius_song(id: i64) -> Result<Value, Error> {
    let res = send_genius_request(format!("songs/{}?text_format=html", id)).await?;
    Ok(res["response"]["song"].clone())
}

pub async fn genius_description(song: &Value) -> Result<String, regex::Error> {
    let mut text = parse_html(song["description"]["html"].as_str().unwrap());

    // Fix weird triple greater-than signs
    let re = Regex::new(r">\n>\n?")?;
    text = re.replace_all(&text, "").to_string();

    // Remove occasional <img> tags since they're not rendered
    let re = Regex::new(r"<img.* />")?;
    text = re.replace_all(&text, "").to_string();

    // Remove dividers because they do not work on embeds
    let re = Regex::new(r"\n?---\n?")?;
    text = re.replace_all(&text, "").to_string();

    Ok(text)
}

pub async fn genius_lyrics(url: &str) -> Result<Vec<String>, Error> {
    let client = reqwest::Client::new();
    let res = client
        .get(url)
        .header(COOKIE, "_genius_ab_test_cohort=33") // Why this?
        .send()
        .await?;

    let document = res.text().await?;
    let fragment = Html::parse_document(&document);
    let selector = Selector::parse(LYRICS_SELECTOR).unwrap();

    let lyrics: Vec<String> = fragment
        .select(&selector)
        .map(|elem| elem.text())
        .flatten()
        .map(ToString::to_string)
        .collect();

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
