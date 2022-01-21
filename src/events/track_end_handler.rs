use serenity::{
    async_trait,
    model::id::GuildId,
    prelude::{Mutex, RwLock, TypeMap},
};
use songbird::{Call, Event, EventContext, EventHandler};
use std::sync::Arc;

use crate::settings::GuildSettingsMap;

pub struct TrackEndNotifier {
    pub guild_id: GuildId,
    pub call: Arc<Mutex<Call>>,
    pub ctx_data: Arc<RwLock<TypeMap>>,
}

#[async_trait]
impl EventHandler for TrackEndNotifier {
    async fn act(&self, _ctx: &EventContext<'_>) -> Option<Event> {
        if GuildSettingsMap::autopause(self.guild_id, &self.ctx_data).await {
            let handler = self.call.lock().await;
            let queue = handler.queue();
            queue.pause().ok();
        }

        None
    }
}
