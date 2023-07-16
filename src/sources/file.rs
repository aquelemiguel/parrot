use crate::commands::play::QueryType;
use ffprobe::FfProbe;
//use serenity::futures::stream::iter;
use serenity::json::Value;
use serenity::model::prelude::Attachment;
use serenity::{self, async_trait};
use songbird::input::{
    error::Result as SongbirdResult,
    error::{Error, Result},
    restartable::Restart,
    Codec, Container, Input, Metadata, Restartable,
};
use std::iter;
use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::process::Command as TokioCommand;
use url::Url;

use crate::sources::ffmpeg;

pub struct FileSource {}

impl FileSource {
    pub fn extract(query: Attachment) -> Option<QueryType> {
        Some(QueryType::File(query))
    }
}

pub struct FileRestartable {}

impl FileRestartable {
    pub async fn download<P: AsRef<str> + Send + Clone + Sync + 'static>(
        uri: P,
        lazy: bool,
    ) -> SongbirdResult<Restartable> {
        Restartable::new(FileRestarter { uri }, lazy).await
    }
}

pub struct FileRestarter<P>
where
    P: AsRef<str> + Send + Sync,
{
    uri: P,
}

#[async_trait]
impl<P> Restart for FileRestarter<P>
where
    P: AsRef<str> + Send + Clone + Sync,
{
    async fn call_restart(&mut self, time: Option<Duration>) -> SongbirdResult<Input> {
        let url = self.uri.as_ref();

        let Some(time) = time else {
            let metadata = _file_metadata(url).await?;
            return from_uri(url, metadata, &[]).await;
        };

        let ts = format!("{:.3}", time.as_secs_f64());
        from_uri(url, Metadata::default(), &["-ss", &ts]).await
    }

    async fn lazy_init(&mut self) -> SongbirdResult<(Option<Metadata>, Codec, Container)> {
        let url = self.uri.as_ref();
        _file_metadata(url)
            .await
            .map(|m| (Some(m), Codec::FloatPcm, Container::Raw))
    }
}

async fn _file_metadata(url: &str) -> SongbirdResult<Metadata> {
    let url_parsed = Url::parse(url).unwrap();
    let res = ffprobe_async_config_url(false, url_parsed.clone())
        .await
        .unwrap();

    let asdf = serde_json::to_string(&res).unwrap();
    let json_res = asdf.as_str();

    let val: Value = serde_json::from_str(json_res).unwrap();
    // tracing::warn!("ffprobe result: {:?}", val);

    let mut metadata = Metadata::from_ffprobe_json(&val);
    // tracing::warn!("metadata: {:?}", metadata);

    metadata.source_url = Some(url.to_string());
    metadata.title = Some(
        url_parsed
            .path_segments()
            .unwrap()
            .last()
            .unwrap()
            .to_string(),
    );

    Ok(metadata)
}

pub async fn from_uri(uri: &str, metadata: Metadata, pre_args: &[&str]) -> Result<Input> {
    // let data = download(uri).await.unwrap();

    let child = Command::new("curl")
        .arg("-s")
        .arg(uri)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    ffmpeg::ffmpeg(child, metadata, pre_args).await
}

/// Run ffprobe with a custom config in async taking a url
/// See [`ConfigBuilder`] for more details.
pub async fn ffprobe_async_config_url(count_frames: bool, url: Url) -> SongbirdResult<FfProbe> {
    let url = url.as_ref();
    let mut args = vec![
        "-v",
        "quiet",
        "-show_format",
        "-show_streams",
        "-print_format",
        "json",
    ];
    if count_frames {
        args.extend(iter::once("-count_frames"));
    }
    args.extend(iter::once(url));

    let mut cmd = TokioCommand::new("ffprobe");

    let out = cmd.args(args).output().await.map_err(|x| Error::Io(x))?;

    serde_json::from_slice::<FfProbe>(&out.stdout).map_err(|x| Error::Json {
        error: x,
        parsed_text: String::from_utf8_lossy(&out.stdout).to_string(),
    })
}
