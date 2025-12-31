use crate::{
    commands::skip::{create_skip_response, force_skip_top_track},
    connection::get_voice_channel_for_user,
    errors::{verify, ParrotError},
    guild::cache::GuildCacheMap,
    messaging::message::ParrotMessage,
    utils::create_response,
};
use serenity::{
    all::CommandInteraction,
    client::Context,
    model::id::GuildId,
    prelude::{Mentionable, RwLock, TypeMap},
};
use std::{collections::HashSet, sync::Arc};

pub async fn voteskip(
    ctx: &Context,
    interaction: &mut CommandInteraction,
) -> Result<(), ParrotError> {
    let guild_id = interaction.guild_id.ok_or(ParrotError::Other(
        "This command can only be used in a server",
    ))?;
    let guild = ctx
        .cache
        .guild(guild_id)
        .ok_or(ParrotError::Other("Guild not found in cache"))?
        .clone();
    let bot_channel_id = get_voice_channel_for_user(&guild, &ctx.cache.current_user().id)
        .ok_or(ParrotError::NotConnected)?;
    let manager = songbird::get(ctx)
        .await
        .ok_or(ParrotError::Other("Voice manager not configured"))?;
    let call = manager.get(guild_id).ok_or(ParrotError::NotConnected)?;

    let handler = call.lock().await;
    let queue = handler.queue();

    verify(!queue.is_empty(), ParrotError::NothingPlaying)?;

    let mut data = ctx.data.write().await;
    let cache_map = data
        .get_mut::<GuildCacheMap>()
        .ok_or(ParrotError::Other("Guild cache not initialized"))?;

    let cache = cache_map.entry(guild_id).or_default();
    cache.current_skip_votes.insert(interaction.user.id);

    let guild_users = ctx
        .cache
        .guild(guild_id)
        .ok_or(ParrotError::Other("Guild not found in cache"))?
        .voice_states
        .clone();
    let channel_guild_users = guild_users
        .into_values()
        .filter(|v| v.channel_id == Some(bot_channel_id));
    let skip_threshold = channel_guild_users.count() / 2;

    if cache.current_skip_votes.len() >= skip_threshold {
        force_skip_top_track(&handler).await?;
        create_skip_response(ctx, interaction, &handler, 1).await
    } else {
        create_response(
            &ctx.http,
            interaction,
            ParrotMessage::VoteSkip {
                mention: interaction.user.id.mention(),
                missing: skip_threshold - cache.current_skip_votes.len(),
            },
        )
        .await
    }
}

pub async fn forget_skip_votes(data: &Arc<RwLock<TypeMap>>, guild_id: GuildId) -> Result<(), ()> {
    let mut data = data.write().await;

    let cache_map = data.get_mut::<GuildCacheMap>().ok_or(())?;
    let cache = cache_map.get_mut(&guild_id).ok_or(())?;
    cache.current_skip_votes = HashSet::new();

    Ok(())
}
