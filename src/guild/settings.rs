use std::collections::{HashMap, HashSet};

use serenity::{model::id::GuildId, prelude::TypeMapKey};

pub struct GuildSettings {
    pub autopause: bool,
    pub allowed_domains: HashSet<String>,
    pub banned_domains: HashSet<String>,
}

pub trait Update {
    fn set_allowed_domains(&mut self, allowed_str: &str);
    fn set_banned_domains(&mut self, banned_str: &str);
    fn update_domains(&mut self);
}

impl Default for GuildSettings {
    fn default() -> GuildSettings {
        GuildSettings {
            autopause: false,
            allowed_domains: HashSet::from(["youtube.com".to_string()]),
            banned_domains: HashSet::new(),
        }
    }
}

impl Update for GuildSettings {
    fn set_allowed_domains(&mut self, allowed_str: &str) {
        let allowed: HashSet<String> = allowed_str
            .split(';')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();

        self.allowed_domains = allowed;
    }

    fn set_banned_domains(&mut self, banned_str: &str) {
        let banned: HashSet<String> = banned_str
            .split(';')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();

        self.banned_domains = banned;
    }

    fn update_domains(&mut self) {
        if !self.allowed_domains.is_empty() && !self.banned_domains.is_empty() {
            self.banned_domains.clear();
        }

        if self.allowed_domains.is_empty() && self.banned_domains.is_empty() {
            self.allowed_domains.insert(String::from("youtube.com"));
            self.banned_domains.clear();
        }
    }
}

pub struct GuildSettingsMap;

impl TypeMapKey for GuildSettingsMap {
    type Value = HashMap<GuildId, GuildSettings>;
}
