use serenity::{async_trait, http::Http, model::channel::Message};
use songbird::{Event, EventContext, EventHandler as VoiceEventHandler, Songbird};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

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
                self.count.store(0, Ordering::Relaxed); // If the bot is playing, reset the counter
            } else {
                if self.count.fetch_add(1, Ordering::Relaxed) >= 10 {
                    self.message
                        .channel_id
                        .send_message(&self.http, |m| {
                            m.embed(|e| {
                                e.description(
                                    "I've been idle for over 5 minutes, so I'll leave for now.
                            Feel free to summon me back any time!",
                                )
                            })
                        })
                        .await
                        .unwrap();

                    handler
                        .leave()
                        .await
                        .expect("Could not leave voice channel!");
                }
            }
        }
        None
    }
}
