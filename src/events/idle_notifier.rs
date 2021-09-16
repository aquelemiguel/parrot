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
        if self.count.fetch_add(1, Ordering::Relaxed) >= 5 {
            self.message
                .channel_id
                .send_message(&self.http, |m| {
                    m.embed(|e| {
                        e.title("Idle");
                        e.description(
                            "I've been idle for 10 minutes, so I'll leave for now.
                        Feel free to summon me back any time!",
                        )
                    })
                })
                .await
                .unwrap();

            self.manager
                .remove(self.message.guild_id.unwrap())
                .await
                .unwrap();
        }
        None
    }
}
