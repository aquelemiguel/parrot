use crate::{
    commands::{summon::summon, EnqueueType, PlayMode},
    handlers::track_end::update_queue_messages,
    sources::youtube::YouTubeRestartable,
    strings::{PLAY_PLAYLIST, PLAY_QUEUE, PLAY_TOP, SEARCHING, TRACK_DURATION, TRACK_TIME_TO_PLAY},
    utils::{
        create_now_playing_embed, create_response, edit_embed_response, edit_response,
        get_human_readable_timestamp,
    },
};
use serenity::{
    builder::CreateEmbed,
    client::Context,
    model::{id::GuildId, interactions::application_command::ApplicationCommandInteraction},
    prelude::{Mutex, SerenityError},
};
use songbird::{tracks::TrackHandle, Call};
use std::{sync::Arc, time::Duration};

pub async fn play(
    ctx: &Context,
    interaction: &mut ApplicationCommandInteraction,
) -> Result<(), SerenityError> {
    let subcommand_args = interaction.data.options.first().unwrap();

    let args = subcommand_args.options.clone();

    let url = args
        .first()
        .unwrap()
        .value
        .as_ref()
        .unwrap()
        .as_str()
        .unwrap();

    let mode = match subcommand_args.name.as_str() {
        "next" => PlayMode::Next,
        "all" => PlayMode::All,
        "reverse" => PlayMode::Reverse,
        "shuffle" => PlayMode::Shuffle,
        "end" => PlayMode::End,
        _ => PlayMode::End,
    };

    let guild_id = interaction.guild_id.unwrap();
    let manager = songbird::get(ctx).await.unwrap();

    // try to join a voice channel if not in one just yet
    summon(ctx, interaction, false).await?;

    // reply with a temporary message while we fetch the source
    // needed because interactions must be replied within 3s and queueing takes longer
    create_response(&ctx.http, interaction, SEARCHING).await?;

    let enqueue_type = match mode {
        PlayMode::All | PlayMode::Reverse | PlayMode::Shuffle => EnqueueType::Playlist,
        _ if url.contains("youtube.com/playlist?list=") => EnqueueType::Playlist,
        _ if url.starts_with("http") => EnqueueType::Link,
        _ => EnqueueType::Search,
    };

    let call = manager.get(guild_id).unwrap();

    match enqueue_type {
        EnqueueType::Link => enqueue_song(ctx, &call, guild_id, url.to_string(), true, mode).await,
        EnqueueType::Search => {
            enqueue_song(ctx, &call, guild_id, url.to_string(), false, mode).await
        }
        EnqueueType::Playlist => enqueue_playlist(ctx, &call, guild_id, url, mode).await,
    };

    let handler = call.lock().await;
    let queue = handler.queue().current_queue();
    drop(handler);

    if queue.len() > 1 {
        let estimated_time = calculate_time_until_play(&queue, mode).await.unwrap();

        match (enqueue_type, mode) {
            (EnqueueType::Link | EnqueueType::Search, PlayMode::Next) => {
                let track = queue.get(1).unwrap();
                let embed = create_queued_embed(PLAY_TOP, track, estimated_time).await;

                edit_embed_response(&ctx.http, interaction, embed).await?;
            }
            (EnqueueType::Link | EnqueueType::Search, PlayMode::End) => {
                let track = queue.last().unwrap();
                let embed = create_queued_embed(PLAY_QUEUE, track, estimated_time).await;

                edit_embed_response(&ctx.http, interaction, embed).await?;
            }
            (EnqueueType::Playlist, _) => {
                edit_response(&ctx.http, interaction, PLAY_PLAYLIST).await?;
            }
            (_, _) => {}
        }
    } else {
        let track = queue.first().unwrap();
        let embed = create_now_playing_embed(track).await;

        edit_embed_response(&ctx.http, interaction, embed).await?;
    }

    Ok(())
}

async fn calculate_time_until_play(queue: &[TrackHandle], mode: PlayMode) -> Option<Duration> {
    if queue.is_empty() {
        return None;
    }

    let top_track = queue.first()?;
    let top_track_elapsed = top_track.get_info().await.unwrap().position;

    let top_track_duration = match top_track.metadata().duration {
        Some(duration) => duration,
        None => return Some(Duration::MAX),
    };

    match mode {
        PlayMode::Next => Some(top_track_duration - top_track_elapsed),
        _ => {
            let center = &queue[1..queue.len() - 1];
            let livestreams =
                center.len() - center.iter().filter_map(|t| t.metadata().duration).count();

            // if any of the tracks before are livestreams, the new track will never play
            if livestreams > 0 {
                return Some(Duration::MAX);
            }

            let durations = center.iter().fold(Duration::ZERO, |acc, x| {
                acc + x.metadata().duration.unwrap()
            });

            Some(durations + top_track_duration - top_track_elapsed)
        }
    }
}

async fn enqueue_playlist(
    ctx: &Context,
    call: &Arc<Mutex<Call>>,
    guild_id: GuildId,
    uri: &str,
    mode: PlayMode,
) {
    if let Some(urls) = YouTubeRestartable::ytdl_playlist(uri, mode).await {
        for url in urls {
            enqueue_song(ctx, call, guild_id, url.to_string(), true, mode).await;
        }
    }
}

async fn create_queued_embed(
    title: &str,
    track: &TrackHandle,
    estimated_time: Duration,
) -> CreateEmbed {
    let mut embed = CreateEmbed::default();
    let metadata = track.metadata().clone();

    embed.thumbnail(metadata.thumbnail.unwrap());

    embed.field(
        title,
        format!(
            "[**{}**]({})",
            metadata.title.unwrap(),
            metadata.source_url.unwrap()
        ),
        false,
    );

    let footer_text = format!(
        "{}{}\n{}{}",
        TRACK_DURATION,
        get_human_readable_timestamp(metadata.duration),
        TRACK_TIME_TO_PLAY,
        get_human_readable_timestamp(Some(estimated_time))
    );

    embed.footer(|footer| footer.text(footer_text));
    embed
}

async fn enqueue_song(
    ctx: &Context,
    call: &Arc<Mutex<Call>>,
    guild_id: GuildId,
    query: String,
    is_url: bool,
    mode: PlayMode,
) {
    let source_return = if is_url {
        YouTubeRestartable::ytdl(query, true).await
    } else {
        YouTubeRestartable::ytdl_search(query, true).await
    };

    // safeguard against ytdl dying on a private/deleted video and killing the playlist
    let source = match source_return {
        Ok(source) => source,
        Err(_) => return,
    };

    let mut handler = call.lock().await;
    handler.enqueue_source(source.into());
    let queue_snapshot = handler.queue().current_queue();
    drop(handler);

    if let PlayMode::Next = mode {
        if queue_snapshot.len() > 2 {
            let handler = call.lock().await;

            handler.queue().modify_queue(|queue| {
                let mut not_playing = queue.split_off(1);
                not_playing.rotate_right(1);
                queue.append(&mut not_playing);
            });
        }
    }

    update_queue_messages(&ctx.http, &ctx.data, call, guild_id).await;
}
