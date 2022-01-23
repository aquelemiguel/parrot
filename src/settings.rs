use serenity::{model::id::GuildId, prelude::TypeMapKey};
use std::collections::HashMap;

#[derive(Default)]
pub struct GuildSettings {
    pub autopause: bool,
}

pub struct GuildSettingsMap;

impl TypeMapKey for GuildSettingsMap {
    type Value = HashMap<GuildId, GuildSettings>;
}
