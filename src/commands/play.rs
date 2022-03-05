use crate::{
    commands::{skip::force_skip_top_track, summon::summon},
    handlers::track_end::update_queue_messages,
    sources::{
        spotify::{MediaType, Spotify},
        youtube::YouTubeRestartable,
    },
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
    model::interactions::application_command::ApplicationCommandInteraction,
    prelude::{Mutex, SerenityError},
};
use songbird::{
    input::{error::Error, Restartable},
    tracks::TrackHandle,
    Call,
};
use std::{cmp::Ordering, error::Error as StdError, sync::Arc, time::Duration};

#[derive(Clone, Copy)]
pub enum Mode {
    End,
    Next,
    All,
    Reverse,
    Shuffle,
    Jump,
}

#[derive(Clone, Copy)]
pub enum QueryType {
    Keywords,
    VideoLink,
    PlaylistLink,
}

pub async fn play(
    ctx: &Context,
    interaction: &mut ApplicationCommandInteraction,
) -> Result<(), SerenityError> {
    let args = interaction.data.options.clone();
    let first_arg = args.first().unwrap();

    let mode = match first_arg.name.as_str() {
        "next" => Mode::Next,
        "all" => Mode::All,
        "reverse" => Mode::Reverse,
        "shuffle" => Mode::Shuffle,
        "jump" => Mode::Jump,
        _ => Mode::End,
    };

    let url = match mode {
        Mode::End => first_arg.value.as_ref().unwrap().as_str().unwrap(),
        _ => first_arg
            .options
            .first()
            .unwrap()
            .value
            .as_ref()
            .unwrap()
            .as_str()
            .unwrap(),
    };

    let guild_id = interaction.guild_id.unwrap();
    let manager = songbird::get(ctx).await.unwrap();

    // try to join a voice channel if not in one just yet
    summon(ctx, interaction, false).await?;
    let call = manager.get(guild_id).unwrap();

    // reply with a temporary message while we fetch the source
    // needed because interactions must be replied within 3s and queueing takes longer
    create_response(&ctx.http, interaction, SEARCHING).await?;

    if url.contains("spotify.com") {
        let spotify = Spotify::auth().await.unwrap();
        let (media_type, media_id) = Spotify::parse(url);

        match media_type {
            MediaType::Track => {
                if let Ok(query) = Spotify::get_track_info(&spotify, media_id).await {
                    println!("{:?}", query);

                    enqueue_track(&call, query, QueryType::Keywords)
                        .await
                        .unwrap();

                    update_queue_messages(&ctx.http, &ctx.data, &call, guild_id).await;
                }
            }
            MediaType::Album => {
                if let Ok(query_list) = Spotify::get_album_info(&spotify, media_id).await {
                    println!("{:?}", query_list);

                    for query in query_list.into_iter() {
                        enqueue_track(&call, query, QueryType::Keywords)
                            .await
                            .unwrap();
                        update_queue_messages(&ctx.http, &ctx.data, &call, guild_id).await;
                    }
                }
            }
            MediaType::Playlist => {
                if let Ok(query_list) = Spotify::get_playlist_info(&spotify, media_id).await {
                    println!("{:?}", query_list);

                    for query in query_list.into_iter() {
                        enqueue_track(&call, query, QueryType::Keywords)
                            .await
                            .unwrap();
                        update_queue_messages(&ctx.http, &ctx.data, &call, guild_id).await;
                    }
                }
            }
        }

        return Ok(());
    }

    let query_type = if url.contains("youtube.com/playlist?list=") {
        QueryType::PlaylistLink
    } else if url.starts_with("http") {
        QueryType::VideoLink
    } else {
        QueryType::Keywords
    };

    let handler = call.lock().await;
    let queue_was_empty = handler.queue().is_empty();
    drop(handler);

    match mode {
        Mode::End => match query_type {
            QueryType::Keywords | QueryType::VideoLink => {
                enqueue_track(&call, url.to_string(), query_type)
                    .await
                    .unwrap();
                update_queue_messages(&ctx.http, &ctx.data, &call, guild_id).await;
            }
            QueryType::PlaylistLink => {
                if let Some(urls) = YouTubeRestartable::ytdl_playlist(url, mode).await {
                    for url in urls.iter() {
                        enqueue_track(&call, url.to_string(), QueryType::VideoLink)
                            .await
                            .unwrap();
                        update_queue_messages(&ctx.http, &ctx.data, &call, guild_id).await;
                    }
                }
            }
        },
        Mode::Next => match query_type {
            QueryType::Keywords | QueryType::VideoLink => {
                insert_track(&call, url.to_string(), query_type, 1)
                    .await
                    .unwrap();
                update_queue_messages(&ctx.http, &ctx.data, &call, guild_id).await;
            }
            QueryType::PlaylistLink => {
                if let Some(urls) = YouTubeRestartable::ytdl_playlist(url, mode).await {
                    for (idx, url) in urls.into_iter().enumerate() {
                        insert_track(&call, url, QueryType::VideoLink, idx + 1)
                            .await
                            .unwrap();
                        update_queue_messages(&ctx.http, &ctx.data, &call, guild_id).await;
                    }
                }
            }
        },
        Mode::Jump => match query_type {
            QueryType::Keywords | QueryType::VideoLink => {
                enqueue_track(&call, url.to_string(), query_type)
                    .await
                    .unwrap();

                if !queue_was_empty {
                    rotate_tracks(&call, 1).await;
                    force_skip_top_track(&call.lock().await).await;
                }

                update_queue_messages(&ctx.http, &ctx.data, &call, guild_id).await;
            }
            QueryType::PlaylistLink => {
                if let Some(urls) = YouTubeRestartable::ytdl_playlist(url, mode).await {
                    let mut insert_idx = 1;
                    let mut is_first_playlist_track = true;

                    for url in urls.into_iter() {
                        insert_track(&call, url, QueryType::VideoLink, insert_idx)
                            .await
                            .unwrap();

                        if is_first_playlist_track && !queue_was_empty {
                            force_skip_top_track(&call.lock().await).await;
                            is_first_playlist_track = false;
                        } else {
                            insert_idx += 1;
                        }

                        update_queue_messages(&ctx.http, &ctx.data, &call, guild_id).await;
                    }
                }
            }
        },
        Mode::All | Mode::Reverse | Mode::Shuffle => match query_type {
            QueryType::VideoLink | QueryType::PlaylistLink => {
                if let Some(urls) = YouTubeRestartable::ytdl_playlist(url, mode).await {
                    for url in urls.into_iter() {
                        enqueue_track(&call, url, QueryType::VideoLink)
                            .await
                            .unwrap();
                        update_queue_messages(&ctx.http, &ctx.data, &call, guild_id).await;
                    }
                }
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

    match queue.len().cmp(&1) {
        Ordering::Greater => {
            let estimated_time = calculate_time_until_play(&queue, mode).await.unwrap();

            match (query_type, mode) {
                (QueryType::VideoLink | QueryType::Keywords, Mode::Next) => {
                    let track = queue.get(1).unwrap();
                    let embed = create_queued_embed(PLAY_TOP, track, estimated_time).await;

                    edit_embed_response(&ctx.http, interaction, embed).await?;
                }
                (QueryType::VideoLink | QueryType::Keywords, Mode::End) => {
                    let track = queue.last().unwrap();
                    let embed = create_queued_embed(PLAY_QUEUE, track, estimated_time).await;

                    edit_embed_response(&ctx.http, interaction, embed).await?;
                }
                (QueryType::PlaylistLink, _) => {
                    edit_response(&ctx.http, interaction, PLAY_PLAYLIST).await?;
                }
                (_, _) => {}
            }
        }
        Ordering::Equal => {
            let track = queue.first().unwrap();
            let embed = create_now_playing_embed(track).await;

            edit_embed_response(&ctx.http, interaction, embed).await?;
        }
        _ => unreachable!(),
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

async fn calculate_time_until_play(queue: &[TrackHandle], mode: Mode) -> Option<Duration> {
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
        Mode::Next => Some(top_track_duration - top_track_elapsed),
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

async fn get_track_source(query: String, query_type: QueryType) -> Result<Restartable, Error> {
    match query_type {
        QueryType::VideoLink => YouTubeRestartable::ytdl(query, true).await,
        QueryType::Keywords => YouTubeRestartable::ytdl_search(query, true).await,
        QueryType::PlaylistLink => unreachable!(),
    }
}

async fn enqueue_track(
    call: &Arc<Mutex<Call>>,
    query: String,
    query_type: QueryType,
) -> Result<(), Box<dyn StdError>> {
    // safeguard against ytdl dying on a private/deleted video and killing the playlist
    let source = get_track_source(query, query_type).await?;

    let mut handler = call.lock().await;
    handler.enqueue_source(source.into());
    Ok(())
}

async fn insert_track(
    call: &Arc<Mutex<Call>>,
    query: String,
    query_type: QueryType,
    idx: usize,
) -> Result<(), Box<dyn StdError>> {
    let handler = call.lock().await;
    let queue_size = handler.queue().len();
    drop(handler);

    if idx == 0 || idx >= queue_size {
        return Err(SerenityError::NotInRange("index", idx as u64, 1, queue_size as u64).into());
    }

    enqueue_track(call, query, query_type).await?;

    let handler = call.lock().await;
    handler.queue().modify_queue(|queue| {
        let back = queue.pop_back().unwrap();
        queue.insert(idx, back);
    });

    Ok(())
}
