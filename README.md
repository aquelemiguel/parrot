# 🦜
A hassle-free, highly performant and fast evolving Discord music bot built with Serenity in Rust.

## Deployment

### Usage
Just [create a bot account](https://discordpy.readthedocs.io/en/stable/discord.html), copy its token into the `DISCORD_TOKEN` environment variable within `.env`.

### Docker

For the hassle free deployment:

```shell
docker run -d --env-file .env ghcr.io/aquelemiguel/parrot:latest
```

## Development

After installing the requirements below, simply run `cargo run`.

### Linux / MacOS
The command below installs a C compiler, GNU autotools, Opus (Discord's audio codec), youtube-dl and FFmpeg.


```shell
apt install build-essential autoconf automake libtool m4 libopus-dev youtube-dl ffmpeg
```

### Windows
If you are using the MSVC toolchain, a prebuilt DLL for Opus is already provided for you.  
You will only need to download [FFmpeg](https://ffmpeg.org/download.html), and install youtube-dl which can be done through Python's package manager, pip.
```shell
pip install youtube_dl
```

### Docker

Within the project folder, simply run the following:

```shell
docker build -t parrot .
docker run -d --env-file .env parrot
```
