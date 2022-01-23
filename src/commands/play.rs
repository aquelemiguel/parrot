use std::{sync::Arc, time::Duration};

use crate::{
    commands::{summon::summon, EnqueueType, PlayFlag},
    strings::{
        FAIL_NO_VOICE_CONNECTION, PLAY_PLAYLIST, PLAY_QUEUE, PLAY_TOP, SEARCHING, TRACK_DURATION,
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

use songbird::{input::Restartable, tracks::TrackHandle, Call};
use youtube_dl::{YoutubeDl, YoutubeDlOutput};

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
    let args = interaction.data.options.clone();

    let url = args
        .first()
        .unwrap()
        .value
        .as_ref()
        .unwrap()
        .as_str()
        .unwrap();

    let guild_id = interaction.guild_id.unwrap();
    let manager = songbird::get(ctx).await.unwrap();

    // try to join a voice channel if not in one just yet
    summon(ctx, interaction, false).await?;

    // halt if isn't in a voice channel at this point
    if manager.get(guild_id).is_none() {
        return create_response(&ctx.http, interaction, FAIL_NO_VOICE_CONNECTION).await;
    }

    // reply with a temporary message while we fetch the source
    create_response(&ctx.http, interaction, SEARCHING).await?;

    let call = manager.get(guild_id).unwrap();

    let enqueue_type = if url.contains("youtube.com/playlist?list=") {
        EnqueueType::PLAYLIST
    } else if url.starts_with("http") {
        EnqueueType::URI
    } else {
        EnqueueType::SEARCH
    };

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
                PlayFlag::PLAYTOP => {
                    let track = queue.get(1).unwrap();
                    let embed = create_queued_embed(PLAY_TOP, track, estimated_time).await;

                    edit_embed_response(&ctx.http, interaction, embed).await?;
                }
                PlayFlag::DEFAULT => {
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

    Ok(())
}

async fn calculate_time_until_play(queue: &[TrackHandle], flag: &PlayFlag) -> Option<Duration> {
    if !queue.is_empty() {
        let top_track = queue.first()?;
        let top_track_elapsed = top_track.get_info().await.unwrap().position;
        let top_track_duration = top_track.metadata().duration?;

        let mut estimated_time = match flag {
            PlayFlag::DEFAULT => queue[1..queue.len() - 1]
                .iter()
                .fold(Duration::ZERO, |acc, x| {
                    acc + x.metadata().duration.unwrap()
                }),
            PlayFlag::PLAYTOP => Duration::ZERO,
        };

        // Add the remaining top track
        estimated_time += top_track_duration - top_track_elapsed;

        Some(estimated_time)
    } else {
        None
    }
}

async fn enqueue_playlist(call: &Arc<Mutex<Call>>, uri: &str) {
    let res = YoutubeDl::new(uri).flat_playlist(true).run().unwrap();

    if let YoutubeDlOutput::Playlist(playlist) = res {
        let entries = playlist.entries.unwrap();

        for entry in entries.iter() {
            let url = format!(
                "https://www.youtube.com/watch?v={}",
                entry.url.as_ref().unwrap()
            );
            enqueue_song(call, url, true, &PlayFlag::DEFAULT).await;
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
        get_human_readable_timestamp(metadata.duration.unwrap()),
        TRACK_TIME_TO_PLAY,
        get_human_readable_timestamp(estimated_time)
    );

    embed.footer(|footer| footer.text(footer_text));
    embed
}

async fn enqueue_song(call: &Arc<Mutex<Call>>, query: String, is_url: bool, flag: &PlayFlag) {
    let source = if is_url {
        Restartable::ytdl(query, true).await.unwrap()
    } else {
        Restartable::ytdl_search(query, false).await.unwrap()
    };

    let mut handler = call.lock().await;
    handler.enqueue_source(source.into());
    let queue_snapshot = handler.queue().current_queue();
    drop(handler);

    if let PlayFlag::PLAYTOP = flag {
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
