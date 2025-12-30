use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serenity::{model::id::GuildId, prelude::TypeMapKey};
use std::{
    collections::{HashMap, HashSet},
    env,
    fs::{create_dir_all, rename, OpenOptions},
    io::{BufReader, BufWriter},
    path::Path,
};

use crate::errors::ParrotError;

const DEFAULT_SETTINGS_PATH: &str = "data/settings";
const DEFAULT_ALLOWED_DOMAINS: [&str; 2] = ["youtube.com", "youtu.be"];

lazy_static! {
    static ref SETTINGS_PATH: String =
        env::var("SETTINGS_PATH").unwrap_or(DEFAULT_SETTINGS_PATH.to_string());
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GuildSettings {
    pub guild_id: GuildId,
    pub autopause: bool,
    pub allowed_domains: HashSet<String>,
    pub banned_domains: HashSet<String>,
}

impl GuildSettings {
    pub fn new(guild_id: GuildId) -> GuildSettings {
        let allowed_domains: HashSet<String> = DEFAULT_ALLOWED_DOMAINS
            .iter()
            .map(|d| d.to_string())
            .collect();

        GuildSettings {
            guild_id,
            autopause: false,
            allowed_domains,
            banned_domains: HashSet::new(),
        }
    }

    pub fn load_if_exists(&mut self) -> Result<(), ParrotError> {
        let path = format!("{}/{}.json", SETTINGS_PATH.as_str(), self.guild_id);
        if !Path::new(&path).exists() {
            return Ok(());
        }
        self.load()
    }

    pub fn load(&mut self) -> Result<(), ParrotError> {
        let path = format!("{}/{}.json", SETTINGS_PATH.as_str(), self.guild_id);
        let file = OpenOptions::new().read(true).open(path)?;
        let reader = BufReader::new(file);
        *self = serde_json::from_reader::<_, GuildSettings>(reader)?;
        Ok(())
    }

    pub fn save(&self) -> Result<(), ParrotError> {
        create_dir_all(SETTINGS_PATH.as_str())?;
        let path = format!("{}/{}.json", SETTINGS_PATH.as_str(), self.guild_id);
        let temp_path = format!("{}/{}.json.tmp", SETTINGS_PATH.as_str(), self.guild_id);

        // Write to temporary file first
        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(&temp_path)?;

        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, self)?;

        // Atomically rename temp file to final path
        rename(&temp_path, &path)?;
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
            self.allowed_domains = DEFAULT_ALLOWED_DOMAINS
                .iter()
                .map(|d| d.to_string())
                .collect();

            self.banned_domains.clear();
        }
    }
}

pub struct GuildSettingsMap;

impl TypeMapKey for GuildSettingsMap {
    type Value = HashMap<GuildId, GuildSettings>;
}
