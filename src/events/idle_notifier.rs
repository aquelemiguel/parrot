use serenity::{async_trait, http::Http, model::channel::Message};
use songbird::{Event, EventContext, EventHandler as VoiceEventHandler, Songbird};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use crate::{strings::IDLE_ALERT, utils::send_simple_message};

pub struct IdleNotifier {
    pub message: Message,
    pub manager: Arc<Songbird>,
    pub count: Arc<AtomicUsize>,
    pub http: Arc<Http>,
}

#[async_trait]
impl VoiceEventHandler for IdleNotifier {
    async fn act(&self, _ctx: &EventContext<'_>) -> Option<Event> {
        let guild_id = self.message.guild_id.unwrap();

        if let Some(lock) = self.manager.get(guild_id) {
            let mut handler = lock.lock().await;

            if !handler.queue().is_empty() {
                // if the bot is playing, reset the counter
                self.count.store(0, Ordering::Relaxed);
            } else if self.count.fetch_add(1, Ordering::Relaxed) >= 10 {
                send_simple_message(&self.http, &self.message, IDLE_ALERT)
                    .await
                    .unwrap();
                handler.leave().await.unwrap();
            }
        }
        None
    }
}
