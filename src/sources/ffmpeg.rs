use songbird::input::{
    error::{Error, Result},
    Codec, Container, Input, Metadata, Reader,
};
use std::process::{Child, Command, Stdio};

pub async fn ffmpeg_yt(mut yt: Child, metadata: Metadata, pre_args: &[&str]) -> Result<Input> {
    let ffmpeg_args = [
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

    let taken_stdout = yt.stdout.take().ok_or(Error::Stdout)?;

    let ffmpeg = Command::new("ffmpeg")
        .args(pre_args)
        .arg("-i")
        .arg("-")
        .args(&ffmpeg_args)
        .stdin(taken_stdout)
        .stderr(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()?;

    let reader = Reader::from(vec![yt, ffmpeg]);

    let input = Input::new(
        true,
        reader,
        Codec::FloatPcm,
        Container::Raw,
        Some(metadata),
    );

    Ok(input)
}
