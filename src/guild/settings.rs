use serenity::{model::id::GuildId, prelude::TypeMapKey};
use std::collections::{HashMap, HashSet};

const DEFAULT_ALLOWED_DOMAINS: [&str; 1] = ["youtube.com"];

pub struct GuildSettings {
    pub locale: String,
    pub autopause: bool,
    pub allowed_domains: HashSet<String>,
    pub banned_domains: HashSet<String>,
}

impl Default for GuildSettings {
    fn default() -> GuildSettings {
        let allowed_domains: HashSet<String> = DEFAULT_ALLOWED_DOMAINS
            .iter()
            .map(|d| d.to_string())
            .collect();

        GuildSettings {
            locale: "en_us".to_owned(),
            autopause: false,
            allowed_domains,
            banned_domains: HashSet::new(),
        }
    }
}

impl GuildSettings {
    pub fn set_allowed_domains(&mut self, allowed_str: &str) {
        let allowed = allowed_str
            .split(';')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();

        self.allowed_domains = allowed;
    }

    pub fn set_banned_domains(&mut self, banned_str: &str) {
        let banned = banned_str
            .split(';')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();

        self.banned_domains = banned;
    }

    pub fn update_domains(&mut self) {
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
