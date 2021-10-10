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
#[aliases("pt")]
async fn playtop(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
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

    if let Some(handler_lock) = manager.get(guild.id) {
        let mut handler = handler_lock.lock().await;

        let queue = handler.queue().current_queue();

        // Check if there are any songs in the queue/currently playing
        if handler.queue().len() > 1 {
            //Shift current songs to the right and
            queue.insert(1, 5);
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
