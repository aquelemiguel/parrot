use std::collections::HashMap;

use serenity::{model::id::GuildId, prelude::TypeMapKey};

#[derive(Default)]
pub struct GuildSettings {
    pub autopause: bool,
}

pub struct GuildSettingsMap;

impl TypeMapKey for GuildSettingsMap {
    type Value = HashMap<GuildId, GuildSettings>;
}
