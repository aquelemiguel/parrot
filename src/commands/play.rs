use crate::{
    commands::{summon::summon, EnqueueType, PlayFlag},
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
    model::interactions::application_command::ApplicationCommandInteraction,
    prelude::{Mutex, SerenityError},
};
use songbird::{tracks::TrackHandle, Call};
use std::{sync::Arc, time::Duration};

pub async fn play(
    ctx: &Context,
    interaction: &mut ApplicationCommandInteraction,
) -> Result<(), SerenityError> {
    _play(ctx, interaction, &PlayFlag::DEFAULT).await
}

pub async fn _play(
    ctx: &Context,
    interaction: &mut ApplicationCommandInteraction,
    flag: &PlayFlag,
) -> Result<(), SerenityError> {
    let subcommand_args = interaction.data.options
        .first()
        .unwrap();

    let args = subcommand_args.options.clone();

    let url = args.first()
        .unwrap()
        .value
        .as_ref()
        .unwrap()
        .as_str()
        .unwrap();

    let flag = match subcommand_args.name.as_str() {
        "next" => &PlayFlag::NEXT,
        "all" => &PlayFlag::ALL,
        _ => flag,
    };

    let guild_id = interaction.guild_id.unwrap();
    let manager = songbird::get(ctx).await.unwrap();

    // try to join a voice channel if not in one just yet
    summon(ctx, interaction, false).await?;

    // reply with a temporary message while we fetch the source
    // needed because interactions must be replied within 3s and queueing takes longer
    create_response(&ctx.http, interaction, SEARCHING).await?;

    let enqueue_type = match flag {
        PlayFlag::ALL => EnqueueType::PLAYLIST,
        _ => {
            if url.contains("youtube.com/playlist?list=") {
                EnqueueType::PLAYLIST
            } else if url.starts_with("http") {
                EnqueueType::URI
            } else {
                EnqueueType::SEARCH
            }
        },
    };

    let call = manager.get(guild_id).unwrap();

    match enqueue_type {
        EnqueueType::URI => enqueue_song(&call, url.to_string(), true, flag).await,
        EnqueueType::SEARCH => enqueue_song(&call, url.to_string(), false, flag).await,
        EnqueueType::PLAYLIST => enqueue_playlist(&call, url).await,
    };

    let handler = call.lock().await;
    let queue = handler.queue().current_queue();
    drop(handler);

    if queue.len() > 1 {
        let estimated_time = calculate_time_until_play(&queue, flag).await.unwrap();

        match enqueue_type {
            EnqueueType::URI | EnqueueType::SEARCH => match flag {
                PlayFlag::NEXT => {
                    let track = queue.get(1).unwrap();
                    let embed = create_queued_embed(PLAY_TOP, track, estimated_time).await;

                    edit_embed_response(&ctx.http, interaction, embed).await?;
                }
                _ => {
                    let track = queue.last().unwrap();
                    let embed = create_queued_embed(PLAY_QUEUE, track, estimated_time).await;

                    edit_embed_response(&ctx.http, interaction, embed).await?;
                }
            },
            EnqueueType::PLAYLIST => {
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

async fn calculate_time_until_play(queue: &[TrackHandle], flag: &PlayFlag) -> Option<Duration> {
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
        PlayFlag::NEXT => Some(top_track_duration - top_track_elapsed),
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

async fn enqueue_playlist(call: &Arc<Mutex<Call>>, uri: &str) {
    if let Some(urls) = YouTubeRestartable::ytdl_playlist(uri).await {
        for url in urls.iter() {
            enqueue_song(call, url.to_string(), true, &PlayFlag::DEFAULT).await;
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

async fn enqueue_song(call: &Arc<Mutex<Call>>, query: String, is_url: bool, flag: &PlayFlag) {
    let source = if is_url {
        YouTubeRestartable::ytdl(query, true).await.unwrap()
    } else {
        YouTubeRestartable::ytdl_search(query, true).await.unwrap()
    };

    let mut handler = call.lock().await;
    handler.enqueue_source(source.into());
    let queue_snapshot = handler.queue().current_queue();
    drop(handler);

    if let PlayFlag::NEXT = flag {
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
