use std::{cmp::min, time::Duration};

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
        let mut tracks = handler.queue().current_queue();

        // If the queue is empty, end the command.
        if tracks.is_empty() {
            send_simple_message(&ctx.http, msg, QUEUE_IS_EMPTY).await;
            return Ok(());
        }

        let top_track = tracks.remove(0);

        let mut message = msg
            .channel_id
            .send_message(&ctx.http, |m| {
                m.embed(|e| create_queue_embed(e, &author_username, &top_track, &tracks, 0));

                if tracks.len() > PAGE_SIZE {
                    m.reactions(vec![ReactionType::Unicode("▶️".to_string())]);
                }

                m
            })
            .await?;

        // If the queue only has the currently playing song, end the command.
        if tracks.is_empty() {
            return Ok(());
        }

        drop(handler); // Release the handler for other commands to use it.

        let mut current_page: usize = 0;
        let mut stream = message
            .await_reactions(&ctx)
            .timeout(Duration::from_secs(60 * 60)) // Stop collection reactions after an hour.
            .author_id(author_id) // Only collect reactions from the invoker.
            .await;

        while let Some(reaction) = stream.next().await {
            let handler = call.lock().await;
            let emoji = &reaction.as_inner_ref().emoji;

            // Refetch the queue in case it changed.
            let mut tracks = handler.queue().current_queue();

            // If the queue is now empty, stop handling reactions.
            if tracks.len() == 0 {
                message.delete_reactions(&ctx.http).await?;

                message
                    .edit(&ctx, |m| {
                        m.embed(|e| e.description(format!("**{}**", QUEUE_IS_EMPTY)))
                    })
                    .await?;

                break;
            }

            let top_track = tracks.remove(0);
            let max_page = tracks.len() / PAGE_SIZE;

            match emoji.as_data().as_str() {
                "◀️" => {
                    message.delete_reactions(&ctx.http).await?;
                    current_page = min(current_page.saturating_sub(1), max_page);

                    message
                        .edit(&ctx, |m| {
                            m.embed(|e| {
                                create_queue_embed(
                                    e,
                                    &author_username,
                                    &top_track,
                                    &tracks,
                                    current_page,
                                )
                            })
                        })
                        .await?;

                    // If we're on the first page, we can't navigate to previous.
                    if current_page > 0 {
                        message
                            .react(&ctx.http, ReactionType::Unicode("◀️".to_string()))
                            .await?;
                    } else {
                        message
                            .delete_reaction_emoji(
                                &ctx.http,
                                ReactionType::Unicode("◀️".to_string()),
                            )
                            .await?;
                    }

                    // If there's enough songs for another page, allow navigating to it.
                    if current_page + 1 <= max_page {
                        message
                            .react(&ctx.http, ReactionType::Unicode("▶️".to_string()))
                            .await?;
                    }
                }
                "▶️" => {
                    message.delete_reactions(&ctx.http).await.unwrap();
                    current_page = min(current_page.saturating_add(1), max_page);

                    message
                        .edit(&ctx, |m| {
                            m.embed(|e| {
                                create_queue_embed(
                                    e,
                                    &author_username,
                                    &top_track,
                                    &tracks,
                                    current_page,
                                )
                            })
                        })
                        .await?;

                    message
                        .react(&ctx.http, ReactionType::Unicode("◀️".to_string()))
                        .await?;

                    if current_page + 1 <= max_page {
                        message
                            .react(&ctx.http, ReactionType::Unicode("▶️".to_string()))
                            .await?;
                    } else {
                        message
                            .delete_reaction_emoji(
                                &ctx.http,
                                ReactionType::Unicode("▶️".to_string()),
                            )
                            .await?;
                    }
                }
                _ => (),
            };
        }

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
    top_track: &TrackHandle,
    queue: &Vec<TrackHandle>,
    page: usize,
) -> &'a mut CreateEmbed {
    embed.title("Queue");

    let metadata = top_track.metadata();
    embed.thumbnail(top_track.metadata().thumbnail.as_ref().unwrap());

    let description = format!(
        "[{}]({}) • `{}`",
        metadata.title.as_ref().unwrap(),
        metadata.source_url.as_ref().unwrap(),
        get_human_readable_timestamp(metadata.duration.unwrap())
    );

    embed.field("🔊  Now playing", description, false);

    if !queue.is_empty() {
        embed.field("⌛  Up next", build_queue_page(queue, page), false);
    }

    embed.footer(|f| {
        f.text(format!(
            "Page {} of {} • Requested by {}",
            page + 1,
            queue.len() / PAGE_SIZE + 1,
            author
        ))
    })
}

fn build_queue_page(tracks: &Vec<TrackHandle>, page: usize) -> String {
    let mut description = String::new();
    let start_idx = PAGE_SIZE * page;

    for (i, t) in tracks.iter().skip(start_idx).take(PAGE_SIZE).enumerate() {
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
