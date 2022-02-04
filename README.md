<p align="center">
  <img alt="Light" src="./docs/logo.png" width="50%">
</p>

<p align="center">
  A hassle-free, highly performant, host-it-yourself Discord music bot
</p>

## Deployment

### Usage
Just [create a bot account](https://github.com/aquelemiguel/parrot/wiki/Create-Your-Discord-Bot), and copy its **token** and **application id** to a `.env` with the `DISCORD_TOKEN` and `DISCORD_APPID` environment variables respectively.

### Docker

```shell
docker run -d --env-file .env ghcr.io/aquelemiguel/parrot:latest
```

## Development

Make sure you've installed Rust. You can install Rust and its package manager, `cargo` by following the instructions on https://rustup.rs/.
After installing the requirements below, simply run `cargo run`.

### Linux/MacOS
The commands below installs a C compiler, GNU autotools and FFmpeg, as well as [yt-dlp](https://github.com/yt-dlp/yt-dlp) through Python's package manager, pip.

#### Linux

```shell
apt install build-essential autoconf automake libtool ffmpeg
pip install -U yt-dlp
```

#### MacOS

```shell
brew install autoconf automake libtool ffmpeg
pip install -U yt-dlp
```

### Windows

If you are using the MSVC toolchain, a prebuilt DLL for Opus is already provided for you.  
You will only need to download [FFmpeg](https://ffmpeg.org/download.html), and install [yt-dlp](https://github.com/yt-dlp/yt-dlp) which can be done through Python's package manager, pip.
```shell
pip install -U yt-dlp
```

If you are using Windows Subsystem for Linux (WSL), you should follow the [Linux/MacOS](#linux--macos) guide, and, in addition to the other required packages, install pkg-config, which you may do by running:

```shell
apt install pkg-config
```

## Testing

Tests are available inside the `src/tests` folder. They can be ran via `cargo test`. It's recommended that you run the tests before submitting your Pull Request.
Increasing the test coverage is also welcome.

### Docker

Within the project folder, simply run the following:

```shell
docker build -t parrot .
docker run -d --env-file .env parrot
```
