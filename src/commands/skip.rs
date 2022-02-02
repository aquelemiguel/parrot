use crate::{
    guild::cache::GuildCacheMap,
    strings::{NOTHING_IS_PLAYING, SKIPPED, SKIP_VOTE_EMOJI, SKIP_VOTE_MISSING, SKIP_VOTE_USER},
    utils::create_response,
};
use serenity::{
    client::Context,
    model::{id::GuildId, interactions::application_command::ApplicationCommandInteraction},
    prelude::{Mentionable, RwLock, SerenityError, TypeMap},
};
use std::sync::Arc;

pub async fn skip(
    ctx: &Context,
    interaction: &mut ApplicationCommandInteraction,
) -> Result<(), SerenityError> {
    let guild_id = interaction.guild_id.unwrap();
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

    if !cache.current_skip_votes.contains(&interaction.user.id) {
        cache.current_skip_votes.push(interaction.user.id);
    }

    let guild = ctx.cache.guild(guild_id).await.unwrap();
    let skip_threshold = guild.voice_states.len() / 2;

    let message = if cache.current_skip_votes.len() >= skip_threshold && queue.skip().is_ok() {
        SKIPPED.to_string()
    } else {
        format!(
            "{}{} {} {} {}",
            SKIP_VOTE_EMOJI,
            interaction.user.id.mention(),
            SKIP_VOTE_USER,
            skip_threshold - cache.current_skip_votes.len(),
            SKIP_VOTE_MISSING
        )
    };

    create_response(&ctx.http, interaction, &message).await?;
    drop(data);

    Ok(())
}

pub async fn forget_skip_votes(data: &Arc<RwLock<TypeMap>>, guild_id: GuildId) {
    let mut data = data.write().await;
    let cache_map = data.get_mut::<GuildCacheMap>().unwrap();

    let cache = cache_map.get_mut(&guild_id).unwrap();
    cache.current_skip_votes = Vec::new();
}
