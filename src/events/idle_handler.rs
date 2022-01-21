use serenity::{
    async_trait,
    http::Http,
    model::{id::GuildId, interactions::application_command::ApplicationCommandInteraction},
    prelude::{Mutex, RwLock, TypeMap},
};
use songbird::{Call, Event, EventContext, EventHandler, Songbird};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use crate::{settings::GuildSettingsMap, strings::IDLE_ALERT};

pub struct IdleHandler {
    pub http: Arc<Http>,
    pub manager: Arc<Songbird>,
    pub interaction: ApplicationCommandInteraction,
    pub limit: usize,
    pub count: Arc<AtomicUsize>,
}

#[async_trait]
impl EventHandler for IdleHandler {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(track_list) = ctx {
            if !track_list.is_empty() {
                self.count.store(0, Ordering::Relaxed);
                return None;
            }

            if self.count.fetch_add(1, Ordering::Relaxed) >= self.limit {
                let guild_id = self.interaction.guild_id?;

                if let Some(call) = self.manager.get(guild_id) {
                    let mut handler = call.lock().await;

                    if handler.leave().await.is_ok() {
                        self.interaction
                            .channel_id
                            .say(&self.http, IDLE_ALERT)
                            .await
                            .unwrap();
                    }
                }
            }
        }
        None
    }
}

pub struct SongEndNotifier {
    pub guild_id: GuildId,
    pub call: Arc<Mutex<Call>>,
    pub ctx_data: Arc<RwLock<TypeMap>>,
}

#[async_trait]
impl EventHandler for SongEndNotifier {
    async fn act(&self, _ctx: &EventContext<'_>) -> Option<Event> {
        if GuildSettingsMap::autopause(self.guild_id, &self.ctx_data).await {
            let handler = self.call.lock().await;
            let queue = handler.queue();
            queue.pause().ok();
        }

        None
    }
}
