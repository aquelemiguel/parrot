use serenity::{
    model::id::GuildId,
    prelude::{RwLock, TypeMap, TypeMapKey},
};
use std::{collections::HashMap, sync::Arc};

#[derive(Default)]
pub struct GuildSettings {
    pub autopause: bool,
}

pub struct GuildSettingsMap;

impl TypeMapKey for GuildSettingsMap {
    type Value = HashMap<GuildId, GuildSettings>;
}

impl GuildSettingsMap {
    pub async fn autopause(guild_id: GuildId, ctx_data: &Arc<RwLock<TypeMap>>) -> bool {
        let reader = ctx_data.read().await;
        let settings = reader.get::<Self>().unwrap();
        settings
            .get(&guild_id)
            .map(|guild_settings| guild_settings.autopause)
            .unwrap_or_default()
    }
}
