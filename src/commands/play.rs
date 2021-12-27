use std::sync::Arc;

use crate::{
    commands::{now_playing::now_playing, summon::summon, EnqueueType, PlayFlag},
    strings::{MISSING_PLAY_QUERY, NO_VOICE_CONNECTION},
    utils::{send_added_to_queue_message, send_simple_message},
};

use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
    prelude::Mutex,
};

use songbird::{input::Restartable, Call};
use youtube_dl::{YoutubeDl, YoutubeDlOutput};

#[command]
#[aliases("p")]
async fn play(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    execute_play(ctx, msg, args, &PlayFlag::DEFAULT).await?;
    Ok(())
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
    summon(ctx, msg, args.clone()).await?;

    // Halt if isn't in a voice channel at this point
    if manager.get(guild.id).is_none() {
        send_simple_message(&ctx.http, msg, NO_VOICE_CONNECTION).await;
        return Ok(());
    }

    let call = manager.get(guild.id).unwrap();

    let enqueue_type = if url.clone().contains("youtube.com/playlist?list=") {
        EnqueueType::PLAYLIST
    } else if url.clone().starts_with("http") {
        EnqueueType::URI
    } else {
        EnqueueType::SEARCH
    };

    match enqueue_type {
        EnqueueType::URI => enqueue_song(&call, url, true, flag).await,
        EnqueueType::SEARCH => {
            let query = String::from(args.rewind().rest()); // Rewind and fetch the entire query
            enqueue_song(&call, query, false, flag).await
        }
        EnqueueType::PLAYLIST => enqueue_playlist(&call, &url).await,
    };

    let handler = call.lock().await;
    let queue = handler.queue().current_queue();
    drop(handler);

    // Send response message
    if queue.len() > 1 {
        match enqueue_type {
            EnqueueType::URI | EnqueueType::SEARCH => match flag {
                PlayFlag::PLAYTOP => {
                    let track = queue.get(1).unwrap();
                    send_added_to_queue_message(&ctx.http, msg, "Added to top", &queue, track)
                        .await;
                }
                PlayFlag::DEFAULT => {
                    let track = queue.last().unwrap();
                    send_added_to_queue_message(&ctx.http, msg, "Added to queue", &queue, track)
                        .await;
                }
            },
            EnqueueType::PLAYLIST => {
                // TODO: Make this a little more informative in the future.
                send_simple_message(&ctx.http, msg, "Added playlist to queue!").await;
            }
        }
    } else {
        now_playing(ctx, msg, args.clone()).await?;
    }

    Ok(())
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
