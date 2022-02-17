use crate::{
    commands::{summon::summon, PlayMode, QueryType},
    handlers::track_end::update_queue_messages,
    sources::youtube::YouTubeRestartable,
    strings::{
        PLAY_ALL_FAILED, PLAY_PLAYLIST, PLAY_QUEUE, PLAY_TOP, SEARCHING, TRACK_DURATION,
        TRACK_TIME_TO_PLAY,
    },
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
        "jump" => PlayMode::Jump,
        _ => PlayMode::End,
    };

    let guild_id = interaction.guild_id.unwrap();
    let manager = songbird::get(ctx).await.unwrap();

    // try to join a voice channel if not in one just yet
    summon(ctx, interaction, false).await?;

    // reply with a temporary message while we fetch the source
    // needed because interactions must be replied within 3s and queueing takes longer
    create_response(&ctx.http, interaction, SEARCHING).await?;

    let query_type = if url.contains("youtube.com/playlist?list=") {
        QueryType::PlaylistLink
    } else if url.starts_with("http") {
        QueryType::VideoLink
    } else {
        QueryType::Keywords
    };

    let call = manager.get(guild_id).unwrap();

    let handler = call.lock().await;
    let queue_was_empty = handler.queue().is_empty();
    drop(handler);

    match mode {
        PlayMode::End => match query_type {
            QueryType::Keywords | QueryType::VideoLink => {
                enqueue_song(&ctx, &call, guild_id, url.to_string(), query_type).await;
            }
            QueryType::PlaylistLink => {
                enqueue_playlist(&ctx, &call, guild_id, url, mode, query_type).await;
            }
        },
        PlayMode::Next => match query_type {
            QueryType::Keywords | QueryType::VideoLink => {
                enqueue_song(&ctx, &call, guild_id, url.to_string(), query_type).await;
                rotate_tracks(&call, 1).await;
            }
            QueryType::PlaylistLink => {
                if let Some(playlist) =
                    enqueue_playlist(&ctx, &call, guild_id, url, mode, query_type).await
                {
                    rotate_tracks(&call, playlist.len()).await;
                }
            }
        },
        PlayMode::Jump => match query_type {
            QueryType::Keywords | QueryType::VideoLink => {
                enqueue_song(&ctx, &call, guild_id, url.to_string(), query_type).await;

                if !queue_was_empty {
                    rotate_tracks(&call, 1).await;
                    force_skip_top_track(&call).await;
                }
            }
            QueryType::PlaylistLink => {
                if let Some(playlist) =
                    enqueue_playlist(&ctx, &call, guild_id, url, mode, query_type).await
                {
                    if !queue_was_empty {
                        rotate_tracks(&call, playlist.len()).await;
                        force_skip_top_track(&call).await;
                    }
                }
            }
        },
        PlayMode::All | PlayMode::Reverse | PlayMode::Shuffle => match query_type {
            QueryType::VideoLink | QueryType::PlaylistLink => {
                enqueue_playlist(&ctx, &call, guild_id, url, mode, query_type).await;
            }
            _ => {
                edit_response(&ctx.http, interaction, PLAY_ALL_FAILED).await?;
                return Ok(());
            }
        },
    }

    let handler = call.lock().await;
    let queue = handler.queue().current_queue();
    drop(handler);

    if queue.len() > 1 {
        let estimated_time = calculate_time_until_play(&queue, mode).await.unwrap();

        match (query_type, mode) {
            (QueryType::VideoLink | QueryType::Keywords, PlayMode::Next) => {
                let track = queue.get(1).unwrap();
                let embed = create_queued_embed(PLAY_TOP, track, estimated_time).await;

                edit_embed_response(&ctx.http, interaction, embed).await?;
            }
            (QueryType::VideoLink | QueryType::Keywords, PlayMode::End) => {
                let track = queue.last().unwrap();
                let embed = create_queued_embed(PLAY_QUEUE, track, estimated_time).await;

                edit_embed_response(&ctx.http, interaction, embed).await?;
            }
            (QueryType::PlaylistLink, _) => {
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

async fn rotate_tracks(call: &Arc<Mutex<Call>>, n: usize) {
    let handler = call.lock().await;

    if handler.queue().len() <= 2 {
        return;
    }

    handler.queue().modify_queue(|queue| {
        let mut not_playing = queue.split_off(1);
        not_playing.rotate_right(n);
        queue.append(&mut not_playing);
    });
}

async fn force_skip_top_track(call: &Arc<Mutex<Call>>) {
    let handler = call.lock().await;

    // this is an odd sequence of commands to ensure the queue is properly updated
    // apparently, skipping/stopping a track takes a little to remove it from the queue
    // also, manually removing tracks doesn't trigger the next track to play
    // so first, stop the top song, manually remove it and then resume playback
    handler.queue().current().unwrap().stop().ok();
    handler.queue().dequeue(0);
    handler.queue().resume().ok();
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
    query_type: QueryType,
) -> Option<Vec<String>> {
    if let Some(urls) = YouTubeRestartable::ytdl_playlist(uri, mode).await {
        for url in urls.iter() {
            enqueue_song(ctx, call, guild_id, url.to_string(), query_type).await;
        }
        return Some(urls);
    }
    None
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
    query_type: QueryType,
) {
    let source_return = match query_type {
        QueryType::VideoLink => YouTubeRestartable::ytdl(query, true).await,
        QueryType::Keywords => YouTubeRestartable::ytdl_search(query, true).await,
        QueryType::PlaylistLink => unreachable!(),
    };

    // safeguard against ytdl dying on a private/deleted video and killing the playlist
    let source = match source_return {
        Ok(source) => source,
        Err(_) => return,
    };

    let mut handler = call.lock().await;
    handler.enqueue_source(source.into());
    drop(handler);

    update_queue_messages(&ctx.http, &ctx.data, call, guild_id).await;
}
