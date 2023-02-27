use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{BufReader, BufWriter},
};

use serenity::{model::id::GuildId, prelude::TypeMapKey};

use crate::errors::ParrotError;

const DEFAULT_ALLOWED_DOMAINS: [&str; 1] = ["youtube.com"];

#[derive(Deserialize, Serialize)]
pub struct GuildSettings {
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
            autopause: false,
            allowed_domains,
            banned_domains: HashSet::new(),
        }
    }
}

impl GuildSettings {
    pub fn from_file(path: &str) -> Result<GuildSettings, ParrotError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let guild_settings = serde_json::from_reader(reader)?;
        Ok(guild_settings)
    }

    pub fn save(&self) -> Result<(), ParrotError> {
        let path = "test.json";
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, self)?;
        Ok(())
    }

    pub fn toggle_autopause(&mut self) {
        self.autopause = !self.autopause;
    }

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
