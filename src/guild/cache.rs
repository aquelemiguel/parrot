use std::{collections::HashMap, sync::Arc};

use serenity::{
    model::{
        channel::Message,
        id::{GuildId, UserId},
    },
    prelude::{RwLock, TypeMapKey},
};

type QueueMessage = (Message, Arc<RwLock<usize>>);

#[derive(Default)]
pub struct GuildCache {
    pub queue_messages: Vec<QueueMessage>,
    pub current_skip_votes: Vec<UserId>,
}

pub struct GuildCacheMap;

impl TypeMapKey for GuildCacheMap {
    type Value = HashMap<GuildId, GuildCache>;
}
