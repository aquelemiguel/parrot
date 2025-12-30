use crate::commands::play::QueryType;
use crate::errors::ParrotError;
use ffprobe::FfProbe;
use serenity::json::Value;
use serenity::model::prelude::Attachment;
use serenity::{self, async_trait};
use songbird::input::{
    error::Result as SongbirdResult,
    error::{Error, Result},
    restartable::Restart,
    Codec, Container, Input, Metadata, Reader, Restartable,
};
use std::process::{Command, Stdio};
use std::time::Duration;
use tempfile::NamedTempFile;
use tokio::io::AsyncWriteExt;
use tokio::process::Command as TokioCommand;
use url::Url;

/// Maximum file size allowed for attachments (50 MB)
const MAX_FILE_SIZE: u64 = 50 * 1024 * 1024;

/// Allowed MIME type prefixes for audio/video files
const ALLOWED_CONTENT_TYPES: &[&str] = &[
    "audio/",
    "video/mp4",
    "video/webm",
    "video/ogg",
    "application/ogg",
];

#[allow(clippy::result_large_err)]
pub fn validate_attachment(attachment: &Attachment) -> std::result::Result<(), ParrotError> {
    // Check file size
    if attachment.size > MAX_FILE_SIZE {
        return Err(ParrotError::FileTooLarge);
    }

    // Check content type
    if let Some(ref content_type) = attachment.content_type {
        let is_allowed = ALLOWED_CONTENT_TYPES
            .iter()
            .any(|ct| content_type.starts_with(ct));

        if !is_allowed {
            return Err(ParrotError::UnsupportedFileType);
        }
    }

    Ok(())
}

#[allow(clippy::result_large_err)]
pub fn extract_query_type(attachment: Attachment) -> std::result::Result<QueryType, ParrotError> {
    validate_attachment(&attachment)?;
    Ok(QueryType::File(attachment))
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
            let metadata = file_metadata(url).await?;
            return create_input_from_uri(url, metadata, &[]).await;
        };

        let ts = format!("{:.3}", time.as_secs_f64());
        create_input_from_uri(url, Metadata::default(), &["-ss", &ts]).await
    }

    async fn lazy_init(&mut self) -> SongbirdResult<(Option<Metadata>, Codec, Container)> {
        let url = self.uri.as_ref();
        file_metadata(url)
            .await
            .map(|m| (Some(m), Codec::FloatPcm, Container::Raw))
    }
}

async fn file_metadata(url: &str) -> SongbirdResult<Metadata> {
    let url_parsed = Url::parse(url).map_err(|e| Error::Json {
        error: serde_json::Error::io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Invalid URL: {}", e),
        )),
        parsed_text: url.to_string(),
    })?;

    let ffprobe_result = run_ffprobe(&url_parsed).await?;

    let json_value: Value = serde_json::to_value(&ffprobe_result).map_err(|e| Error::Json {
        error: e,
        parsed_text: String::new(),
    })?;

    let mut metadata = Metadata::from_ffprobe_json(&json_value);

    metadata.source_url = Some(url.to_string());

    // Extract filename from URL path safely
    if let Some(mut segments) = url_parsed.path_segments() {
        if let Some(last_segment) = segments.next_back() {
            if !last_segment.is_empty() {
                metadata.title = Some(last_segment.to_string());
            }
        }
    }

    Ok(metadata)
}

#[allow(clippy::io_other_error)]
async fn download_file(url: &str) -> Result<Vec<u8>> {
    let response = reqwest::get(url).await.map_err(|e| {
        Error::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to download file: {}", e),
        ))
    })?;

    // Check content length if available
    if let Some(content_length) = response.content_length() {
        if content_length > MAX_FILE_SIZE {
            return Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "File too large",
            )));
        }
    }

    let bytes = response.bytes().await.map_err(|e| {
        Error::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to read response bytes: {}", e),
        ))
    })?;

    // Double-check size after download
    if bytes.len() as u64 > MAX_FILE_SIZE {
        return Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "File too large",
        )));
    }

    Ok(bytes.to_vec())
}

async fn create_input_from_uri(uri: &str, metadata: Metadata, pre_args: &[&str]) -> Result<Input> {
    let data = download_file(uri).await?;

    // Create a temporary file (automatically cleaned up when dropped)
    let temp_file = NamedTempFile::new().map_err(Error::Io)?;
    let temp_path = temp_file.path().to_string_lossy().to_string();

    // Write data to temp file
    {
        let mut file = tokio::fs::File::create(&temp_path)
            .await
            .map_err(Error::Io)?;
        file.write_all(&data).await.map_err(Error::Io)?;
        file.sync_all().await.map_err(Error::Io)?;
    }

    // Build ffmpeg arguments - read directly from file instead of using cat
    let ffmpeg_args = [
        "-i",
        &temp_path,
        "-f",
        "s16le",
        "-ac",
        "2",
        "-ar",
        "48000",
        "-acodec",
        "pcm_f32le",
        "-",
    ];

    // Use std::process::Command for compatibility with songbird's Reader
    let ffmpeg = Command::new("ffmpeg")
        .args(pre_args)
        .args(ffmpeg_args)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(Error::Io)?;

    let reader = Reader::from(vec![ffmpeg]);

    let input = Input::new(
        true,
        reader,
        Codec::FloatPcm,
        Container::Raw,
        Some(metadata),
    );

    // Keep temp_file alive - it will be cleaned up when this function returns
    // and the file handle is dropped. The ffmpeg process has already opened it.
    std::mem::forget(temp_file);

    Ok(input)
}

async fn run_ffprobe(url: &Url) -> SongbirdResult<FfProbe> {
    let url_str = url.as_str();
    let args = vec![
        "-v",
        "quiet",
        "-show_format",
        "-show_streams",
        "-print_format",
        "json",
        url_str,
    ];

    let output = TokioCommand::new("ffprobe")
        .args(&args)
        .output()
        .await
        .map_err(Error::Io)?;

    serde_json::from_slice::<FfProbe>(&output.stdout).map_err(|e| Error::Json {
        error: e,
        parsed_text: String::from_utf8_lossy(&output.stdout).to_string(),
    })
}
