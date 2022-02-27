use crate::{
    commands::skip::{create_skip_response, force_skip_top_track},
    connection::get_voice_channel_for_user,
    errors::ParrotError,
    guild::cache::GuildCacheMap,
    strings::{NOTHING_IS_PLAYING, SKIP_VOTE_EMOJI, SKIP_VOTE_MISSING, SKIP_VOTE_USER},
    utils::create_response,
};
use serenity::{
    client::Context,
    model::{id::GuildId, interactions::application_command::ApplicationCommandInteraction},
    prelude::{Mentionable, RwLock, TypeMap},
};
use std::{collections::HashSet, sync::Arc};

pub async fn voteskip(
    ctx: &Context,
    interaction: &mut ApplicationCommandInteraction,
) -> Result<(), ParrotError> {
    let guild_id = interaction.guild_id.unwrap();
    let bot_channel_id = get_voice_channel_for_user(
        &ctx.cache.guild(guild_id).await.unwrap(),
        &ctx.cache.current_user_id().await,
    )
    .unwrap();
    let manager = songbird::get(ctx).await.unwrap();
    let call = manager.get(guild_id).unwrap();

    let handler = call.lock().await;
    let queue = handler.queue();

    if queue.is_empty() {
        return create_response(&ctx.http, interaction, NOTHING_IS_PLAYING).await;
    }

    let mut data = ctx.data.write().await;
    let cache_map = data.get_mut::<GuildCacheMap>().unwrap();

    let cache = cache_map.entry(guild_id).or_default();
    cache.current_skip_votes.insert(interaction.user.id);

    let guild_users = ctx.cache.guild(guild_id).await.unwrap().voice_states;
    let channel_guild_users = guild_users
        .into_values()
        .filter(|v| v.channel_id.unwrap() == bot_channel_id);
    let skip_threshold = channel_guild_users.count() / 2;

    if cache.current_skip_votes.len() >= skip_threshold {
        force_skip_top_track(&handler).await;
        create_skip_response(ctx, interaction, &handler, 1).await
    } else {
        create_response(
            &ctx.http,
            interaction,
            &format!(
                "{}{} {} {} {}",
                SKIP_VOTE_EMOJI,
                interaction.user.id.mention(),
                SKIP_VOTE_USER,
                skip_threshold - cache.current_skip_votes.len(),
                SKIP_VOTE_MISSING
            ),
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
