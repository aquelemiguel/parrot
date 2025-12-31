use crate::commands::play::{Mode, QueryType};
use serde_json::Value;
use songbird::input::{AuxMetadata, Compose, Input, YoutubeDl};
use std::io::BufRead;
use std::process::Stdio;
use std::sync::OnceLock;
use tokio::process::Command as TokioCommand;

static HTTP_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

fn get_http_client() -> reqwest::Client {
    HTTP_CLIENT
        .get_or_init(reqwest::Client::new)
        .clone()
}

pub struct YouTube {}

impl YouTube {
    pub fn extract(query: &str) -> Option<QueryType> {
        if query.contains("list=") {
            Some(QueryType::PlaylistLink(query.to_string()))
        } else {
            Some(QueryType::VideoLink(query.to_string()))
        }
    }
}

pub struct YouTubeRestartable {}

impl YouTubeRestartable {
    /// Creates a YouTube input and fetches its metadata
    pub async fn ytdl<P: AsRef<str> + Send + Clone + Sync + 'static>(
        uri: P,
    ) -> Result<(Input, AuxMetadata), crate::errors::ParrotError> {
        let client = get_http_client();
        let mut source = YoutubeDl::new(client, uri.as_ref().to_string());
        let metadata = source.aux_metadata().await.map_err(|e| {
            crate::errors::ParrotError::TrackFail(format!("Failed to get metadata: {}", e))
        })?;
        Ok((source.into(), metadata))
    }

    /// Creates a YouTube search input and fetches its metadata
    pub async fn ytdl_search<P: AsRef<str> + Send + Clone + Sync + 'static>(
        uri: P,
    ) -> Result<(Input, AuxMetadata), crate::errors::ParrotError> {
        let client = get_http_client();
        let mut source = YoutubeDl::new_search(client, uri.as_ref().to_string());
        let metadata = source.aux_metadata().await.map_err(|e| {
            crate::errors::ParrotError::TrackFail(format!("Failed to get metadata: {}", e))
        })?;
        Ok((source.into(), metadata))
    }

    pub async fn ytdl_playlist(uri: &str, mode: Mode) -> Option<Vec<String>> {
        let mut args = vec![uri, "--flat-playlist", "-j"];
        match mode {
            Mode::Reverse => args.push("--playlist-reverse"),
            Mode::Shuffle => args.push("--playlist-random"),
            _ => {}
        }

        let output = TokioCommand::new("yt-dlp")
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .await
            .ok()?;

        let lines: Vec<String> = output
            .stdout
            .lines()
            .map_while(Result::ok)
            .filter_map(|line| {
                let entry: Value = serde_json::from_str(&line).ok()?;
                entry
                    .get("webpage_url")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            })
            .collect();

        Some(lines)
    }
}
