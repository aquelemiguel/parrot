use serenity::{
    async_trait,
    model::id::GuildId,
    prelude::{Mutex, RwLock, TypeMap},
};
use songbird::{Call, Event, EventContext, EventHandler};
use std::sync::Arc;

use crate::settings::GuildSettingsMap;

pub struct TrackEndHandler {
    pub guild_id: GuildId,
    pub call: Arc<Mutex<Call>>,
    pub ctx_data: Arc<RwLock<TypeMap>>,
}

#[async_trait]
impl EventHandler for TrackEndHandler {
    async fn act(&self, _ctx: &EventContext<'_>) -> Option<Event> {
        let data = self.ctx_data.read().await;
        let settings = data.get::<GuildSettingsMap>().unwrap();

        let autopause = settings
            .get(&self.guild_id)
            .map(|guild_settings| guild_settings.autopause)
            .unwrap_or_default();

        if autopause {
            let handler = self.call.lock().await;
            let queue = handler.queue();
            queue.pause().ok();
        }

        None
    }
}
