use crate::{commands::PlayMode, sources::ffmpeg::ffmpeg};
use serde_json::Value;
use serenity::async_trait;
use songbird::input::{
    error::{Error as SongbirdError, Result as SongbirdResult},
    restartable::Restart,
    Codec, Container, Input, Metadata, Restartable,
};
use std::{
    io::{BufRead, BufReader, Read},
    process::Command,
    process::{Child, Stdio},
    time::Duration,
};
use tokio::{process::Command as TokioCommand, task};

const NEWLINE_BYTE: u8 = 0xA;

pub struct YouTubeRestartable {}

impl YouTubeRestartable {
    pub async fn ytdl<P: AsRef<str> + Send + Clone + Sync + 'static>(
        uri: P,
        lazy: bool,
    ) -> SongbirdResult<Restartable> {
        Restartable::new(YouTubeRestarter { uri }, lazy).await
    }

    pub async fn ytdl_search<P: AsRef<str> + Send + Clone + Sync + 'static>(
        uri: P,
        lazy: bool,
    ) -> SongbirdResult<Restartable> {
        let uri = format!("ytsearch1:{}", uri.as_ref());
        Restartable::new(YouTubeRestarter { uri }, lazy).await
    }

    pub async fn ytdl_playlist(uri: &str, mode: PlayMode) -> Option<Vec<String>> {
        let mut args = vec![uri, "--flat-playlist", "-j"];
        match mode {
            PlayMode::Reverse => args.push("--playlist-reverse"),
            PlayMode::Shuffle => args.push("--playlist-random"),
            _ => {}
        }

        let mut child = Command::new("yt-dlp")
            .args(args)
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        if let Some(stdout) = &mut child.stdout {
            let reader = BufReader::new(stdout);

            let lines = reader.lines().flatten().map(|line| {
                let entry: Value = serde_json::from_str(&line).unwrap();
                entry.get("url").unwrap().as_str().unwrap().to_string()
            });

            Some(lines.collect())
        } else {
            None
        }
    }
}

struct YouTubeRestarter<P>
where
    P: AsRef<str> + Send + Sync,
{
    uri: P,
}

#[async_trait]
impl<P> Restart for YouTubeRestarter<P>
where
    P: AsRef<str> + Send + Clone + Sync,
{
    async fn call_restart(&mut self, time: Option<Duration>) -> SongbirdResult<Input> {
        let (yt, metadata) = ytdl(self.uri.as_ref()).await?;

        if let Some(time) = time {
            let ts = format!("{:.3}", time.as_secs_f64());
            ffmpeg(yt, metadata, &["-ss", &ts]).await
        } else {
            ffmpeg(yt, metadata, &[]).await
        }
    }

    async fn lazy_init(&mut self) -> SongbirdResult<(Option<Metadata>, Codec, Container)> {
        _ytdl_metadata(self.uri.as_ref())
            .await
            .map(|m| (Some(m), Codec::FloatPcm, Container::Raw))
    }
}

async fn ytdl(uri: &str) -> Result<(Child, Metadata), SongbirdError> {
    let ytdl_args = [
        "-j",            // print JSON information for video for metadata
        "--no-simulate", // ensure video is downloaded regardless of printing
        "-f",
        "bestaudio", // select best quality audio-only
        "-R",
        "infinite",        // infinite number of download retries
        "--no-playlist",   // only download the video if URL also has playlist info
        "--ignore-config", // disable all configuration files for a yt-dlp run
        uri,
        "-o",
        "-", // stream data to stdout
    ];

    let mut yt = Command::new("yt-dlp")
        .args(&ytdl_args)
        .stdin(Stdio::null())
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    // track info json (for metadata) is piped to stderr by design choice of yt-dlp
    // the actual track is streamed to stdout
    let stderr = yt.stderr.take();
    let (returned_stderr, value) = task::spawn_blocking(move || {
        let mut s = stderr.unwrap();
        let out: SongbirdResult<Value> = {
            let mut o_vec = vec![];
            let mut serde_read = BufReader::new(s.by_ref());

            if let Ok(len) = serde_read.read_until(NEWLINE_BYTE, &mut o_vec) {
                serde_json::from_slice(&o_vec[..len]).map_err(|err| SongbirdError::Json {
                    error: err,
                    parsed_text: std::str::from_utf8(&o_vec).unwrap_or_default().to_string(),
                })
            } else {
                Result::Err(SongbirdError::Metadata)
            }
        };

        (s, out)
    })
    .await
    .map_err(|_| SongbirdError::Metadata)?;

    let metadata = Metadata::from_ytdl_output(value?);
    yt.stderr = Some(returned_stderr);

    Ok((yt, metadata))
}

async fn _ytdl_metadata(uri: &str) -> SongbirdResult<Metadata> {
    let ytdl_args = [
        "-j", // print JSON information for video for metadata
        "-R",
        "infinite",        // infinite number of download retries
        "--no-playlist",   // only download the video if URL also has playlist info
        "--ignore-config", // disable all configuration files for a yt-dlp run
        uri,
        "-o",
        "-", // stream data to stdout
    ];

    let youtube_dl_output = TokioCommand::new("yt-dlp")
        .args(&ytdl_args)
        .stdin(Stdio::null())
        .output()
        .await?;

    let o_vec = youtube_dl_output.stderr;

    // read until newline byte
    let end = (&o_vec)
        .iter()
        .position(|el| *el == NEWLINE_BYTE)
        .unwrap_or_else(|| o_vec.len());

    let value = serde_json::from_slice(&o_vec[..end]).map_err(|err| SongbirdError::Json {
        error: err,
        parsed_text: std::str::from_utf8(&o_vec).unwrap_or_default().to_string(),
    })?;

    let metadata = Metadata::from_ytdl_output(value);
    Ok(metadata)
}
