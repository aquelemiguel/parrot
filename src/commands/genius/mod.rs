use html2md::parse_html;
use reqwest::header::{AUTHORIZATION, COOKIE};
use reqwest::Error;
use scraper::{Html, Selector};
use serde_json::Value;
use std::io::Write;
use std::{env, fs::File};

pub mod explain;
pub mod lyrics;

const GENIUS_BASE_ENDPOINT: &str = "https://api.genius.com/";

pub async fn genius_search(query: &str) -> Option<Vec<Value>> {
    let res = send_genius_request(format!("search?q={}", query))
        .await
        .unwrap();

    res["response"]["hits"].as_array().map(|v| v.to_owned())
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

pub async fn get_genius_song_id(query: &str) -> Option<i64> {
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
