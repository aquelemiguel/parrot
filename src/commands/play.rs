use std::sync::Arc;

use crate::{
    commands::{now_playing::now_playing, queue::queue, summon::summon, PlayFlag},
    strings::{MISSING_PLAY_QUERY, NO_VOICE_CONNECTION},
    utils::send_simple_message,
};

use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
    prelude::Mutex,
};

use songbird::{input::Restartable, tracks::TrackHandle, Call};
use youtube_dl::{YoutubeDl, YoutubeDlOutput};

async fn enqueue_song(
    call: &Arc<Mutex<Call>>,
    query: String,
    is_url: bool,
    flag: &PlayFlag,
) -> Vec<TrackHandle> {
    let source = match is_url {
        true => Restartable::ytdl(query, true).await.unwrap(),
        false => Restartable::ytdl_search(query, false).await.unwrap(),
    };

    let mut handler = call.lock().await;
    handler.enqueue_source(source.into());
    let mut queue_snapshot = handler.queue().current_queue();
    drop(handler);

    match flag {
        PlayFlag::PLAYTOP => {
            if queue_snapshot.len() > 2 {
                let handler = call.lock().await;

                handler.queue().modify_queue(|queue| {
                    let mut not_playing = queue.split_off(1);
                    not_playing.rotate_right(1);
                    queue.append(&mut not_playing);
                });

                queue_snapshot = handler.queue().current_queue();
            }
        }
        _ => (),
    }

    queue_snapshot
}

async fn enqueue_playlist(
    call: &Arc<Mutex<Call>>,
    uri: &String,
    flag: &PlayFlag,
) -> Vec<TrackHandle> {
    let mut entries = vec![];

    match YoutubeDl::new(uri).flat_playlist(true).run() {
        Ok(result) => {
            if let YoutubeDlOutput::Playlist(playlist) = result {
                entries = playlist.entries.unwrap();

                for entry in &entries {
                    let uri = format!(
                        "https://www.youtube.com/watch?v={}",
                        entry.url.as_ref().unwrap()
                    );
                    let source = Restartable::ytdl(uri, true).await.unwrap();
                    let mut handler = call.lock().await;
                    handler.enqueue_source(source.into());
                }
            }
        }
        Err(_) => todo!("Show failed to fetch playlist message!"),
    }

    let handler = call.lock().await;
    let mut queue_snapshot = handler.queue().current_queue();
    drop(handler);

    match flag {
        PlayFlag::PLAYTOP => {
            if queue_snapshot.len() > 2 {
                let handler = call.lock().await;

                handler.queue().modify_queue(|queue| {
                    let mut not_playing = queue.split_off(1);
                    let rotations = not_playing.len() - &entries.len();
                    not_playing.rotate_left(rotations);
                    queue.append(&mut not_playing);
                });

                queue_snapshot = handler.queue().current_queue();
            }
        }
        _ => (),
    }

    queue_snapshot
}

pub async fn execute_play(
    ctx: &Context,
    msg: &Message,
    mut args: Args,
    flag: &PlayFlag,
) -> CommandResult {
    // Handle empty requests
    let url = match args.single::<String>() {
        Ok(url) => url,
        Err(_) => {
            send_simple_message(&ctx.http, msg, MISSING_PLAY_QUERY).await;
            return Ok(());
        }
    };

    let guild = msg.guild(&ctx.cache).await.unwrap();
    let manager = songbird::get(ctx)
        .await
        .expect("Could not retrieve Songbird voice client");

    // Try to join a voice channel if not in one just yet
    summon(&ctx, &msg, args.clone()).await?;

    // Halt if isn't in a voice channel at this point
    if manager.get(guild.id).is_none() {
        send_simple_message(&ctx.http, msg, NO_VOICE_CONNECTION).await;
        return Ok(());
    }

    let call = manager.get(guild.id).unwrap();
    let queue;

    if url.clone().contains("youtube.com/playlist?list=") {
        queue = enqueue_playlist(&call, &url, &flag).await;
    } else if url.clone().starts_with("http") {
        queue = enqueue_song(&call, url, true, &flag).await;
    } else {
        let query = String::from(args.rewind().rest()); // Rewind and fetch the entire query
        queue = enqueue_song(&call, query, false, &flag).await;
    }

    // If there's only one song in the queue now, it must be playing
    if queue.len() == 1 {
        now_playing(&ctx, &msg, args.clone()).await?;
    }

    Ok(())
}

#[command]
#[aliases("p")]
async fn play(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    execute_play(ctx, msg, args, &PlayFlag::DEFAULT).await?;

    //     // If it's not going to be played immediately, notify it has been enqueued
    //     if queue.len() > 1 {
    //         let last_track = queue.last().unwrap();
    //         let metadata = last_track.metadata().clone();
    //         let position = last_track.get_info().await?.position;

    //         msg.channel_id
    //             .send_message(&ctx.http, |m| {
    //                 m.embed(|e| {
    //                     e.title("Added to queue");
    //                     e.thumbnail(metadata.thumbnail.unwrap());

    //                     e.description(format!(
    //                         "[**{}**]({})",
    //                         metadata.title.unwrap(),
    //                         metadata.source_url.unwrap()
    //                     ));

    //                     let mut estimated_time = queue
    //                         .into_iter()
    //                         .map(|track| track.metadata().duration.unwrap())
    //                         .sum();

    //                     estimated_time -= position;

    //                     let footer_text = format!(
    //                         "Track duration: {}\nEstimated time until play: {}",
    //                         get_human_readable_timestamp(metadata.duration.unwrap()),
    //                         get_human_readable_timestamp(estimated_time)
    //                     );

    //                     let mut footer = CreateEmbedFooter::default();
    //                     footer.text(footer_text);

    //                     e.set_footer(footer)
    //                 })
    //             })
    //             .await?;
    //     }

    Ok(())
}
