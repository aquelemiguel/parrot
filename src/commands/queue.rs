use crate::{
    commands::play::get_track_metadata,
    errors::ParrotError,
    guild::cache::GuildCacheMap,
    handlers::track_end::ModifyQueueHandler,
    messaging::messages::{
        QUEUE_EXPIRED, QUEUE_NOTHING_IS_PLAYING, QUEUE_NOW_PLAYING, QUEUE_NO_SONGS, QUEUE_PAGE,
        QUEUE_PAGE_OF, QUEUE_UP_NEXT,
    },
    utils::get_human_readable_timestamp,
};
use serenity::{
    all::{
        ButtonStyle, CommandInteraction, CreateActionRow, CreateButton, CreateEmbedFooter,
        CreateInteractionResponse, CreateInteractionResponseMessage,
    },
    builder::CreateEmbed,
    client::Context,
    futures::StreamExt,
    model::{channel::Message, id::GuildId},
    prelude::{RwLock, TypeMap},
};
use songbird::{tracks::TrackHandle, Event, TrackEvent};
use std::{
    cmp::{max, min},
    fmt::Write,
    ops::Add,
    sync::Arc,
    time::Duration,
};

const EMBED_PAGE_SIZE: usize = 6;
const EMBED_TIMEOUT: u64 = 3600;

pub async fn queue(ctx: &Context, interaction: &mut CommandInteraction) -> Result<(), ParrotError> {
    use serenity::all::EditMessage;

    let guild_id = interaction.guild_id.ok_or(ParrotError::Other(
        "This command can only be used in a server",
    ))?;

    let manager = songbird::get(ctx)
        .await
        .ok_or(ParrotError::Other("Voice manager not configured"))?;

    let call = manager.get(guild_id).ok_or(ParrotError::NotConnected)?;

    let handler = call.lock().await;
    let tracks = handler.queue().current_queue();
    drop(handler);

    let num_pages = calculate_num_pages(&tracks);
    let response = CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new()
            .add_embed(create_queue_embed(&tracks, 0))
            .components(vec![build_nav_btns(0, num_pages)]),
    );

    interaction.create_response(&ctx.http, response).await?;

    let mut message = interaction.get_response(&ctx.http).await?;
    let page: Arc<RwLock<usize>> = Arc::new(RwLock::new(0));

    // store this interaction to context.data for later edits
    let mut data = ctx.data.write().await;
    let cache_map = data.get_mut::<GuildCacheMap>().unwrap();

    let cache = cache_map.entry(guild_id).or_default();
    cache.queue_messages.push((message.clone(), page.clone()));
    drop(data);

    // refresh the queue interaction whenever a track ends
    let mut handler = call.lock().await;
    handler.add_global_event(
        Event::Track(TrackEvent::End),
        ModifyQueueHandler {
            http: ctx.http.clone(),
            ctx_data: ctx.data.clone(),
            call: call.clone(),
            guild_id,
        },
    );
    drop(handler);

    let mut collector = message
        .await_component_interactions(ctx)
        .timeout(Duration::from_secs(EMBED_TIMEOUT))
        .stream();

    while let Some(mci) = collector.next().await {
        let btn_id = &mci.data.custom_id;

        // refetch the queue in case it changed
        let handler = call.lock().await;
        let tracks = handler.queue().current_queue();
        drop(handler);

        let num_pages = calculate_num_pages(&tracks);
        let mut page_wlock = page.write().await;

        *page_wlock = match btn_id.as_str() {
            "<<" => 0,
            "<" => min(page_wlock.saturating_sub(1), num_pages - 1),
            ">" => min(page_wlock.add(1), num_pages - 1),
            ">>" => num_pages - 1,
            _ => continue,
        };

        let response = CreateInteractionResponse::UpdateMessage(
            CreateInteractionResponseMessage::new()
                .add_embed(create_queue_embed(&tracks, *page_wlock))
                .components(vec![build_nav_btns(*page_wlock, num_pages)]),
        );
        mci.create_response(&ctx, response).await?;
    }

    let edit = EditMessage::new()
        .embed(CreateEmbed::new().description(QUEUE_EXPIRED))
        .components(vec![]);
    if let Err(e) = message.edit(&ctx.http, edit).await {
        eprintln!("[WARN] Failed to edit queue message: {}", e);
    }

    forget_queue_message(&ctx.data, &mut message, guild_id)
        .await
        .ok();

    Ok(())
}

pub fn create_queue_embed(tracks: &[TrackHandle], page: usize) -> CreateEmbed {
    let (description, thumbnail) = if !tracks.is_empty() {
        let metadata = get_track_metadata(&tracks[0]).unwrap_or_default();
        let desc = format!(
            "[{}]({}) • `{}`",
            metadata.title.as_deref().unwrap_or("Unknown"),
            metadata.source_url.as_deref().unwrap_or("#"),
            get_human_readable_timestamp(metadata.duration)
        );
        (desc, metadata.thumbnail)
    } else {
        (String::from(QUEUE_NOTHING_IS_PLAYING), None)
    };

    let footer_text = format!(
        "{} {} {} {}",
        QUEUE_PAGE,
        page + 1,
        QUEUE_PAGE_OF,
        calculate_num_pages(tracks),
    );

    let mut embed = CreateEmbed::new()
        .field(QUEUE_NOW_PLAYING, &description, false)
        .field(QUEUE_UP_NEXT, build_queue_page(tracks, page), false)
        .footer(CreateEmbedFooter::new(footer_text));

    if let Some(thumb) = thumbnail {
        embed = embed.thumbnail(thumb);
    }

    embed
}

fn build_single_nav_btn(label: &str, is_disabled: bool) -> CreateButton {
    CreateButton::new(label.to_string().to_ascii_lowercase())
        .label(label)
        .style(ButtonStyle::Primary)
        .disabled(is_disabled)
}

pub fn build_nav_btns(page: usize, num_pages: usize) -> CreateActionRow {
    let (cant_left, cant_right) = (page < 1, page >= num_pages - 1);

    CreateActionRow::Buttons(vec![
        build_single_nav_btn("<<", cant_left),
        build_single_nav_btn("<", cant_left),
        build_single_nav_btn(">", cant_right),
        build_single_nav_btn(">>", cant_right),
    ])
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
        let metadata = get_track_metadata(t).unwrap_or_default();
        let title = metadata.title.as_deref().unwrap_or("Unknown");
        let url = metadata.source_url.as_deref().unwrap_or("#");
        let duration = get_human_readable_timestamp(metadata.duration);

        let _ = writeln!(
            description,
            "`{}.` [{}]({}) • `{}`",
            i + start_idx + 1,
            title,
            url,
            duration
        );
    }

    description
}

pub fn calculate_num_pages(tracks: &[TrackHandle]) -> usize {
    let num_pages = ((tracks.len() as f64 - 1.0) / EMBED_PAGE_SIZE as f64).ceil() as usize;
    max(1, num_pages)
}

pub async fn forget_queue_message(
    data: &Arc<RwLock<TypeMap>>,
    message: &mut Message,
    guild_id: GuildId,
) -> Result<(), ()> {
    let mut data_wlock = data.write().await;
    let cache_map = data_wlock.get_mut::<GuildCacheMap>().ok_or(())?;

    let cache = cache_map.get_mut(&guild_id).ok_or(())?;
    cache.queue_messages.retain(|(m, _)| m.id != message.id);

    Ok(())
}
