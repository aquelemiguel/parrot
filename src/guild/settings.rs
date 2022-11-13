use std::collections::{HashMap, HashSet};

use serenity::{model::id::GuildId, prelude::TypeMapKey};

#[derive(Default)]
pub struct GuildSettings {
    pub autopause: bool,
    pub allowed_domains: HashSet<String>,
    pub banned_domains: HashSet<String>,
}

pub struct GuildSettingsMap;

impl TypeMapKey for GuildSettingsMap {
    type Value = HashMap<GuildId, GuildSettings>;
}
