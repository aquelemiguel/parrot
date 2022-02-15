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
use rand::{seq::SliceRandom, thread_rng};
use serenity::{
    builder::CreateEmbed,
    client::Context,
    model::interactions::application_command::ApplicationCommandInteraction,
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

    let flag = match subcommand_args.name.as_str() {
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

    let enqueue_type = match flag {
        PlayMode::All | PlayMode::Reverse | PlayMode::Shuffle => EnqueueType::Playlist,
        _ if url.contains("youtube.com/playlist?list=") => EnqueueType::Playlist,
        _ if url.starts_with("http") => EnqueueType::Link,
        _ => EnqueueType::Search,
    };

    let call = manager.get(guild_id).unwrap();

    match enqueue_type {
        EnqueueType::Link => enqueue_song(&call, url.to_string(), true, &flag).await,
        EnqueueType::Search => enqueue_song(&call, url.to_string(), false, &flag).await,
        EnqueueType::Playlist => enqueue_playlist(&call, url, &flag).await,
    };

    let handler = call.lock().await;
    let queue = handler.queue().current_queue();
    drop(handler);

    if queue.len() > 1 {
        let estimated_time = calculate_time_until_play(&queue, &flag).await.unwrap();

        match enqueue_type {
            EnqueueType::Link | EnqueueType::Search => match flag {
                PlayMode::Next => {
                    let track = queue.get(1).unwrap();
                    let embed = create_queued_embed(PLAY_TOP, track, estimated_time).await;

                    edit_embed_response(&ctx.http, interaction, embed).await?;
                }
                PlayMode::End => {
                    let track = queue.last().unwrap();
                    let embed = create_queued_embed(PLAY_QUEUE, track, estimated_time).await;

                    edit_embed_response(&ctx.http, interaction, embed).await?;
                }
                _ => {}
            },
            EnqueueType::Playlist => {
                edit_response(&ctx.http, interaction, PLAY_PLAYLIST).await?;
            }
        }
    } else {
        let track = queue.first().unwrap();
        let embed = create_now_playing_embed(track).await;

        edit_embed_response(&ctx.http, interaction, embed).await?;
    }

    update_queue_messages(&ctx.http, &ctx.data, &call, guild_id).await;
    Ok(())
}

async fn calculate_time_until_play(queue: &[TrackHandle], flag: &PlayMode) -> Option<Duration> {
    if queue.is_empty() {
        return None;
    }

    let top_track = queue.first()?;
    let top_track_elapsed = top_track.get_info().await.unwrap().position;

    let top_track_duration = match top_track.metadata().duration {
        Some(duration) => duration,
        None => return Some(Duration::MAX),
    };

    match flag {
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

async fn enqueue_playlist(call: &Arc<Mutex<Call>>, uri: &str, flag: &PlayMode) {
    if let Some(urls) = YouTubeRestartable::ytdl_playlist(uri).await {
        let ordered_urls = match flag {
            PlayMode::Reverse => urls.iter().rev().cloned().collect(),
            PlayMode::Shuffle => {
                let mut urls_copy = urls.clone();
                let mut rng = thread_rng();
                urls_copy.shuffle(&mut rng);
                urls_copy
            }
            _ => urls,
        };
        for url in ordered_urls {
            enqueue_song(call, url.to_string(), true, flag).await;
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

async fn enqueue_song(call: &Arc<Mutex<Call>>, query: String, is_url: bool, flag: &PlayMode) {
    let source_return = if is_url {
        YouTubeRestartable::ytdl(query, true).await
    } else {
        YouTubeRestartable::ytdl_search(query, true).await
    };

    // safeguard against ytdl dying on a private / deleted video and killing the playlist
    let source = match source_return {
        Ok(source) => source,
        Err(error) => {
            println!("{}", error);
            return;
        }
    };

    let mut handler = call.lock().await;
    handler.enqueue_source(source.into());
    let queue_snapshot = handler.queue().current_queue();
    drop(handler);

    if let PlayMode::Next = flag {
        if queue_snapshot.len() > 2 {
            let handler = call.lock().await;

            handler.queue().modify_queue(|queue| {
                let mut not_playing = queue.split_off(1);
                not_playing.rotate_right(1);
                queue.append(&mut not_playing);
            });
        }
    }
}
