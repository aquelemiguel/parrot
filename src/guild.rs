use serenity::{
    model::{channel::Message, id::GuildId},
    prelude::{RwLock, TypeMapKey},
};
use std::{collections::HashMap, sync::Arc};

type QueueMessage = (Message, Arc<RwLock<usize>>);

#[derive(Default)]
pub struct GuildCache {
    pub queue_messages: Vec<QueueMessage>,
}

pub struct GuildCacheMap;

impl TypeMapKey for GuildCacheMap {
    type Value = HashMap<GuildId, GuildCache>;
}

#[derive(Default)]
pub struct GuildSettings {
    pub autopause: bool,
}

pub struct GuildSettingsMap;

impl TypeMapKey for GuildSettingsMap {
    type Value = HashMap<GuildId, GuildSettings>;
}
