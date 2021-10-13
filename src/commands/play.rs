use std::{
    sync::{atomic::AtomicUsize, Arc},
    time::Duration,
};

use crate::{
    events::idle_notifier::IdleNotifier,
    strings::{AUTHOR_NOT_FOUND, MISSING_PLAY_QUERY, NO_VOICE_CONNECTION},
    utils::{get_human_readable_timestamp, send_simple_message},
};

use serenity::{
    builder::CreateEmbedFooter,
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};

use songbird::{input::Restartable, Event};

use youtube_dl::{YoutubeDl, YoutubeDlOutput};

#[command]
#[aliases("p")]
async fn play(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
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
    if manager.get(guild.id).is_none() {
        let channel_id = guild
            .voice_states
            .get(&msg.author.id)
            .and_then(|voice_state| voice_state.channel_id);

        // Abort if it cannot find the author in any voice channels
        if channel_id.is_none() {
            send_simple_message(&ctx.http, msg, AUTHOR_NOT_FOUND).await;
            return Ok(());
        } else {
            let lock = manager.join(guild.id, channel_id.unwrap()).await.0;
            lock.lock().await;
        }
    }

    if let Some(handler_lock) = manager.get(guild.id) {
        let mut handler = handler_lock.lock().await;

        // Handle an URL
        if url.clone().starts_with("http") {
            // If is a playlist
            if url.clone().contains("youtube.com/playlist?list=") {
                match YoutubeDl::new(url).flat_playlist(true).run() {
                    Ok(result) => {
                        if let YoutubeDlOutput::Playlist(playlist) = result {
                            let entries = playlist.entries.unwrap();

                            for entry in entries {
                                let uri = format!(
                                    "https://www.youtube.com/watch?v={}",
                                    entry.url.unwrap()
                                );
                                println!("Enqueued {}", uri);
                                let source = Restartable::ytdl(uri, true).await?;
                                handler.enqueue_source(source.into());
                            }
                        }
                    }
                    Err(_) => todo!("Show failed to fetch playlist message!"),
                }
            }
            // Just a single song
            else {
                let source = Restartable::ytdl(url, true).await?;
                handler.enqueue_source(source.into());
            }
        }
        // Play via search
        else {
            let query = args.rewind().remains().unwrap(); // Rewind and fetch the entire query
            let source = Restartable::ytdl_search(query, false).await?;
            handler.enqueue_source(source.into());
        }

        let queue = handler.queue().current_queue();

        // If it's not going to be played immediately, notify it has been enqueued
        if handler.queue().len() > 1 {
            let last_track = queue.last().unwrap();
            let metadata = last_track.metadata().clone();
            let position = last_track.get_info().await?.position;

            msg.channel_id
                .send_message(&ctx.http, |m| {
                    m.embed(|e| {
                        e.title("Added to queue");
                        e.thumbnail(metadata.thumbnail.unwrap());

                        e.description(format!(
                            "[**{}**]({})",
                            metadata.title.unwrap(),
                            metadata.source_url.unwrap()
                        ));

                        let mut estimated_time = queue
                            .into_iter()
                            .map(|track| track.metadata().duration.unwrap())
                            .sum();

                        estimated_time -= position;

                        let footer_text = format!(
                            "Track duration: {}\nEstimated time until play: {}",
                            get_human_readable_timestamp(metadata.duration.unwrap()),
                            get_human_readable_timestamp(estimated_time)
                        );

                        let mut footer = CreateEmbedFooter::default();
                        footer.text(footer_text);

                        e.set_footer(footer)
                    })
                })
                .await?;
        } else {
            let current_track = queue.first().unwrap();
            let metadata = current_track.metadata().clone();

            msg.channel_id
                .send_message(&ctx.http, |m| {
                    m.embed(|e| {
                        e.title("Now playing");
                        e.thumbnail(metadata.thumbnail.unwrap());

                        let title = metadata.title.as_ref().unwrap();
                        let url = metadata.source_url.as_ref().unwrap();
                        e.description(format!("[**{}**]({})", title, url));

                        let duration = metadata.duration.unwrap();
                        let mut footer = CreateEmbedFooter::default();

                        footer.text(format!(
                            "Track duration: {}\nRequested by: {}",
                            get_human_readable_timestamp(duration),
                            msg.author.name
                        ));

                        e.set_footer(footer)
                    })
                })
                .await?;
        }
    } else {
        send_simple_message(&ctx.http, msg, NO_VOICE_CONNECTION).await;
    }

    Ok(())
}
