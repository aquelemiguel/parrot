use crate::{strings::NO_VOICE_CONNECTION, utils::{get_human_readable_timestamp, send_simple_message}};
use serenity::{builder::{CreateEmbed}, client::Context, framework::standard::{macros::command, CommandResult}, futures::{StreamExt, future}, model::channel::{Message, ReactionType}};
use songbird::tracks::TrackHandle;

#[command]
async fn queue(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild(&ctx.cache).await.unwrap().id;
    let manager = songbird::get(ctx).await.expect("Could not retrieve Songbird voice client");
    let author_id = msg.author.id;

    if let Some(call) = manager.get(guild_id) {
        let handler = call.lock().await;
        let tracks = handler.queue().current_queue();

        let mut message = msg.channel_id.send_message(&ctx.http, |m| {
            m.embed(|e| {
                create_queue_embed(e, &tracks, 0)
            });

            if tracks.len() > 9 {
                m.reactions(vec![ReactionType::Unicode("▶️".to_string())]);
            }

            m
        }).await?;

        let ctx = ctx.clone();
        let handler = handler.clone();

        tokio::spawn(async move {
            let mut current_page: usize = 0;
            let mut reactions = message.await_reactions(&ctx).author_id(author_id).await;

            while let Some(reaction) = reactions.next().await {
                let emoji = &reaction.as_inner_ref().emoji;
                let tracks = handler.queue().current_queue();   // Refetch the queue in case it changed. 

                match emoji.as_data().as_str() {
                    "◀️" => {
                        message.delete_reactions(&ctx.http).await.unwrap();

                        message.edit(&ctx, |m| {
                            current_page = current_page.saturating_sub(1);
                            m.embed(|e| create_queue_embed(e, &tracks, current_page))
                        }).await.unwrap();

                        // If we're on the first page, we can't navigate to previous.
                        if current_page == 0 {
                            message.delete_reaction_emoji(&ctx.http, ReactionType::Unicode("◀️".to_string())).await.unwrap();
                        }

                        // If there's enough songs for another page, allow navigating to it.
                        if 1 + (current_page + 1) * 8 <= tracks.len() {
                            message.react(&ctx.http, ReactionType::Unicode("▶️".to_string())).await.unwrap();
                        }
                    },
                    "▶️" => {
                        message.delete_reactions(&ctx.http).await.unwrap();

                        message.edit(&ctx, |m| {
                            current_page = current_page.saturating_add(1);
                            m.embed(|e| create_queue_embed(e, &tracks, current_page))
                        }).await.unwrap();

                        // If the next page exceeds the size of the queue, disable navigating to next page.
                        if 1 + (current_page + 1) * 8 > tracks.len() {
                            message.delete_reaction_emoji(&ctx.http, ReactionType::Unicode("▶️".to_string())).await.unwrap();
                        }

                        message.react(&ctx.http, ReactionType::Unicode("◀️".to_string())).await.unwrap();
                    },
                    _ => ()
                };
            } 
        });
    } else {
        send_simple_message(&ctx.http, msg, NO_VOICE_CONNECTION).await;
    }

    Ok(())
}

pub fn create_queue_embed<'a>(embed: &'a mut CreateEmbed, tracks: &Vec<TrackHandle>, page: usize) -> &'a mut CreateEmbed {
    embed.title("Queue");

    let top_track = tracks.first().unwrap();
    let metadata = top_track.metadata();

    embed.thumbnail(top_track.metadata().thumbnail.as_ref().unwrap());

    let description = format!(
        "[{}]({}) • `{}`",
        metadata.title.as_ref().unwrap(),
        metadata.source_url.as_ref().unwrap(),
        get_human_readable_timestamp(metadata.duration.unwrap())
    );

    embed.field("🔊  Now playing", description, false);

    if tracks.len() > 1 {
        embed.field("⏬  Up next", build_queue_page(tracks, page), false);
    }

    embed.footer(|f| f.text(format!("Page {} of {}", page + 1, (tracks.len() - 1) / 8 + 1)))
}

fn build_queue_page(tracks: &Vec<TrackHandle>, page: usize) -> String {
    let mut description = String::new();
    let start_idx = 1 + 8 * page;

    for (i, t) in tracks.iter().skip(start_idx).take(8).enumerate() {
        let title = t.metadata().title.as_ref().unwrap();
        let url = t.metadata().source_url.as_ref().unwrap();
        let duration = get_human_readable_timestamp(t.metadata().duration.unwrap());

        description.push_str(&format!("`{}.` [{}]({}) • `{}`\n", i + start_idx, title, url, duration));
    }

    description
}
