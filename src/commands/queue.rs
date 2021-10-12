use std::{
    cmp::{max, min},
    time::Duration,
};

use crate::{
    strings::{NO_VOICE_CONNECTION, QUEUE_EXPIRED, QUEUE_IS_EMPTY},
    utils::{get_full_username, get_human_readable_timestamp, send_simple_message},
};
use serenity::{
    builder::CreateEmbed,
    client::Context,
    framework::standard::{macros::command, CommandResult},
    futures::StreamExt,
    model::channel::{Message, ReactionType},
};
use songbird::tracks::TrackHandle;

const PAGE_SIZE: usize = 6;

#[command]
async fn queue(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild(&ctx.cache).await.unwrap().id;
    let manager = songbird::get(ctx)
        .await
        .expect("Could not retrieve Songbird voice client");

    let author_id = msg.author.id;
    let author_username = get_full_username(&msg.author);

    if let Some(call) = manager.get(guild_id) {
        let handler = call.lock().await;
        let tracks = handler.queue().current_queue();

        // If the queue is empty, end the command.
        if tracks.is_empty() {
            send_simple_message(&ctx.http, msg, QUEUE_IS_EMPTY).await;
            return Ok(());
        }

        let reactions = vec!["⏪", "◀️", "▶️", "⏩"]
            .iter()
            .map(|r| ReactionType::Unicode(r.to_string()))
            .collect::<Vec<ReactionType>>();

        let mut message = msg
            .channel_id
            .send_message(&ctx.http, |m| {
                m.embed(|e| create_queue_embed(e, &author_username, &tracks, 0));
                m.reactions(reactions.clone())
            })
            .await?;

        drop(handler); // Release the handler for other commands to use it.

        let mut current_page: usize = 0;
        let mut stream = message
            .await_reactions(&ctx)
            .timeout(Duration::from_secs(60 * 60)) // Stop collecting reactions after an hour.
            .author_id(author_id) // Only collect reactions from the invoker.
            .await;

        while let Some(reaction) = stream.next().await {
            let handler = call.lock().await;
            let emoji = &reaction.as_inner_ref().emoji;

            // Refetch the queue in case it changed.
            let tracks = handler.queue().current_queue();

            // Clean previous reactions.
            message.delete_reactions(&ctx.http).await?;

            for reaction in reactions.clone() {
                message.react(&ctx.http, reaction).await?;
            }

            let num_pages = calculate_num_pages(&tracks);

            current_page = match emoji.as_data().as_str() {
                "⏪" => 0,
                "◀️" => min(current_page.saturating_sub(1), num_pages - 1),
                "▶️" => min(current_page + 1, num_pages - 1),
                "⏩" => num_pages - 1,
                _ => continue,
            };

            message
                .edit(&ctx, |m| {
                    m.embed(|e| create_queue_embed(e, &author_username, &tracks, current_page))
                })
                .await?;
        }

        // If it reaches this point, the stream has expired.
        message.delete_reactions(&ctx.http).await.unwrap();
        message
            .edit(&ctx, |m| {
                m.embed(|e| e.title("Queue").description(QUEUE_EXPIRED))
            })
            .await?;
    } else {
        send_simple_message(&ctx.http, msg, NO_VOICE_CONNECTION).await;
    }

    Ok(())
}

pub fn create_queue_embed<'a>(
    embed: &'a mut CreateEmbed,
    author: &str,
    tracks: &[TrackHandle],
    page: usize,
) -> &'a mut CreateEmbed {
    embed.title("Queue");
    let description;

    if !tracks.is_empty() {
        let metadata = tracks[0].metadata();
        embed.thumbnail(tracks[0].metadata().thumbnail.as_ref().unwrap());

        description = format!(
            "[{}]({}) • `{}`",
            metadata.title.as_ref().unwrap(),
            metadata.source_url.as_ref().unwrap(),
            get_human_readable_timestamp(metadata.duration.unwrap())
        );
    } else {
        description = String::from("Nothing is playing!");
    }

    embed.field("🔊  Now playing", description, false);
    embed.field("⌛  Up next", build_queue_page(tracks, page), false);

    embed.footer(|f| {
        f.text(format!(
            "Page {} of {} • Requested by {}",
            page + 1,
            calculate_num_pages(tracks),
            author
        ))
    })
}

fn build_queue_page(tracks: &[TrackHandle], page: usize) -> String {
    let start_idx = PAGE_SIZE * page;
    let queue: Vec<&TrackHandle> = tracks.iter().skip(start_idx + 1).take(PAGE_SIZE).collect();

    if queue.is_empty() {
        return String::from("There's no songs up next!");
    }

    let mut description = String::new();

    for (i, t) in queue.iter().enumerate() {
        let title = t.metadata().title.as_ref().unwrap();
        let url = t.metadata().source_url.as_ref().unwrap();
        let duration = get_human_readable_timestamp(t.metadata().duration.unwrap());

        description.push_str(&format!(
            "`{}.` [{}]({}) • `{}`\n",
            i + start_idx + 1,
            title,
            url,
            duration
        ));
    }

    description
}

fn calculate_num_pages(tracks: &[TrackHandle]) -> usize {
    let num_pages = ((tracks.len() as f64 - 1.0) / PAGE_SIZE as f64).ceil() as usize;
    max(1, num_pages)
}
