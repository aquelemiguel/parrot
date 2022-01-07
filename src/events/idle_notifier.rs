use serenity::{async_trait, http::Http, model::channel::Message};
use songbird::{Event, EventContext, EventHandler as VoiceEventHandler, Songbird};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use crate::{strings::IDLE_ALERT, utils::send_simple_message};

pub struct IdleNotifier {
    pub http: Arc<Http>,
    pub manager: Arc<Songbird>,
    pub msg: Message,
    pub limit: usize,
    pub count: Arc<AtomicUsize>,
}

#[async_trait]
impl VoiceEventHandler for IdleNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(track_list) = ctx {
            if !track_list.is_empty() {
                self.count.store(0, Ordering::Relaxed);
                return None;
            }

            if self.count.fetch_add(1, Ordering::Relaxed) >= self.limit {
                let guild_id = self.msg.guild_id.unwrap();

                if let Some(call) = self.manager.get(guild_id) {
                    let mut handler = call.lock().await;

                    if handler.leave().await.is_ok() {
                        send_simple_message(&self.http, &self.msg, IDLE_ALERT)
                            .await
                            .unwrap();
                    }
                }
            }
        }
        None
    }
}
