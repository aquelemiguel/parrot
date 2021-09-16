use crate::util::{create_default_embed, get_human_readable_timestamp};
use serenity::{
    builder::CreateEmbedFooter,
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};
use songbird::input::{Input, Restartable};

#[command]
#[aliases("p")]
async fn play(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    // Handle empty requests
    let url = match args.single::<String>() {
        Ok(url) => url,
        Err(_) => {
            msg.channel_id
                .send_message(&ctx.http, |m| {
                    m.embed(|e| {
                        create_default_embed(e, "Play", "Must provide a URL to a video or audio");
                        e
                    })
                })
                .await?;

            return Ok(());
        }
    };

    let guild = msg.guild(&ctx.cache).await.unwrap();
    let manager = songbird::get(ctx).await.expect("").clone();

    if let Some(handler_lock) = manager.get(guild.id) {
        let mut handler = handler_lock.lock().await;
        let source: Restartable;

        // Play via URL
        if url.clone().starts_with("http") {
            source = Restartable::ytdl(url, true).await?;
        }
        // Play via search
        else {
            let query = args.rewind().remains().unwrap(); // Rewind and fetch the entire query
            source = Restartable::ytdl_search(query, true).await?;
        }

        let input: Input = source.into();
        let metadata = input.metadata.clone();

        // If it's not going to be played immediately, notify it has been enqueued
        if !handler.queue().is_empty() {
            let queue = handler.queue().current_queue();
            let top_track_position = queue.first().unwrap().get_info().await?.position;

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

                        let queue = handler.queue().current_queue();

                        let mut estimated_time = queue
                            .into_iter()
                            .map(|track| track.metadata().duration.unwrap())
                            .sum();

                        estimated_time = estimated_time - top_track_position;

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

        handler.enqueue_source(input);
    } else {
        msg.channel_id
            .send_message(&ctx.http, |m| {
                m.embed(|e| {
                    create_default_embed(e, "Play", "Not in a voice channel!");
                    e
                })
            })
            .await?;
    }

    Ok(())
}
