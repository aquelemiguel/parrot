use std::cmp::{max, min};

use crate::{
    strings::{
        FAIL_NO_VOICE_CONNECTION, QUEUE_NOTHING_IS_PLAYING, QUEUE_NOW_PLAYING, QUEUE_NO_SONGS,
        QUEUE_PAGE, QUEUE_PAGE_OF, QUEUE_REQUEST_BY, QUEUE_UP_NEXT,
    },
    utils::{create_response, get_full_username, get_human_readable_timestamp},
};
use serenity::{
    builder::CreateEmbed,
    client::Context,
    futures::StreamExt,
    model::{
        channel::ReactionType,
        interactions::{
            application_command::ApplicationCommandInteraction, message_component::ButtonStyle,
            InteractionResponseType,
        },
    },
    prelude::SerenityError,
};
use songbird::tracks::TrackHandle;

const EMBED_PAGE_SIZE: usize = 6;

pub async fn queue(
    ctx: &Context,
    interaction: &mut ApplicationCommandInteraction,
) -> Result<(), SerenityError> {
    let guild_id = interaction.guild_id.unwrap();
    let manager = songbird::get(ctx).await.unwrap();

    let author_username = get_full_username(&interaction.user);

    let call = match manager.get(guild_id) {
        Some(call) => call,
        None => return create_response(&ctx.http, interaction, FAIL_NO_VOICE_CONNECTION).await,
    };

    let handler = call.lock().await;
    let tracks = handler.queue().current_queue();
    drop(handler);

    interaction
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message
                        .create_embed(|e| create_queue_embed(e, &author_username, &tracks, 0))
                        .components(|components| {
                            components.create_action_row(|action_row| {
                                action_row
                                    .create_button(|button| {
                                        button
                                            .custom_id("⏪".to_string().to_ascii_lowercase())
                                            .emoji(ReactionType::Unicode("⏪".to_string()))
                                            .style(ButtonStyle::Secondary)
                                    })
                                    .create_button(|button| {
                                        button
                                            .custom_id("◀️".to_string().to_ascii_lowercase())
                                            .emoji(ReactionType::Unicode("◀️".to_string()))
                                            .style(ButtonStyle::Secondary)
                                    })
                                    .create_button(|button| {
                                        button
                                            .custom_id("▶️".to_string().to_ascii_lowercase())
                                            .emoji(ReactionType::Unicode("▶️".to_string()))
                                            .style(ButtonStyle::Secondary)
                                    })
                                    .create_button(|button| {
                                        button
                                            .custom_id("⏩".to_string().to_ascii_lowercase())
                                            .emoji(ReactionType::Unicode("⏩".to_string()))
                                            .style(ButtonStyle::Secondary)
                                    })
                            })
                        })
                })
        })
        .await?;

    let message = interaction.get_interaction_response(&ctx.http).await?;

    let mut cib = message.await_component_interactions(&ctx).await;
    let mut current_page: usize = 0;

    while let Some(mci) = cib.next().await {
        let btn_id = &mci.data.custom_id;

        // refetch the queue in case it changed
        let handler = call.lock().await;
        let tracks = handler.queue().current_queue();
        drop(handler);

        let num_pages = calculate_num_pages(&tracks);

        current_page = match btn_id.as_str() {
            "⏪" => 0,
            "◀️" => min(current_page.saturating_sub(1), num_pages - 1),
            "▶️" => min(current_page + 1, num_pages - 1),
            "⏩" => num_pages - 1,
            _ => continue,
        };

        mci.create_interaction_response(&ctx, |r| {
            r.kind(InteractionResponseType::UpdateMessage);
            r.interaction_response_data(|d| {
                d.create_embed(|e| create_queue_embed(e, &author_username, &tracks, current_page))
            })
        })
        .await?;
    }

    Ok(())
}

fn create_queue_embed<'a>(
    embed: &'a mut CreateEmbed,
    author: &str,
    tracks: &[TrackHandle],
    page: usize,
) -> &'a mut CreateEmbed {
    let description = if !tracks.is_empty() {
        let metadata = tracks[0].metadata();
        embed.thumbnail(tracks[0].metadata().thumbnail.as_ref().unwrap());

        format!(
            "[{}]({}) • `{}`",
            metadata.title.as_ref().unwrap(),
            metadata.source_url.as_ref().unwrap(),
            get_human_readable_timestamp(metadata.duration.unwrap())
        )
    } else {
        String::from(QUEUE_NOTHING_IS_PLAYING)
    };

    embed.field(QUEUE_NOW_PLAYING, description, false);
    embed.field(QUEUE_UP_NEXT, build_queue_page(tracks, page), false);
    embed.footer(|f| {
        f.text(format!(
            "{} {} {} {} • {} {}",
            QUEUE_PAGE,
            page + 1,
            QUEUE_PAGE_OF,
            calculate_num_pages(tracks),
            QUEUE_REQUEST_BY,
            author
        ))
    })
}

fn build_queue_page(tracks: &[TrackHandle], page: usize) -> String {
    let start_idx = EMBED_PAGE_SIZE * page;
    let queue: Vec<&TrackHandle> = tracks
        .iter()
        .skip(start_idx + 1)
        .take(EMBED_PAGE_SIZE)
        .collect();

    if queue.is_empty() {
        return String::from(QUEUE_NO_SONGS);
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
    let num_pages = ((tracks.len() as f64 - 1.0) / EMBED_PAGE_SIZE as f64).ceil() as usize;
    max(1, num_pages)
}
