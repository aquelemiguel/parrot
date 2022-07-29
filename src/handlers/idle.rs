use serenity::{
    async_trait, http::Http,
    model::application::interaction::application_command::ApplicationCommandInteraction,
};
use songbird::{tracks::PlayMode, Event, EventContext, EventHandler, Songbird};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use crate::messaging::messages::IDLE_ALERT;

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
            // looks like the track list isn't ordered here, so the first track in the list isn't
            // guaranteed to be the first track in the actual queue, so search the entire list
            let bot_is_playing = track_list
                .iter()
                .any(|track| matches!(track.0.playing, PlayMode::Play));

            // if there's a track playing, then reset the counter
            if bot_is_playing {
                self.count.store(0, Ordering::Relaxed);
                return None;
            }

            if self.count.fetch_add(1, Ordering::Relaxed) >= self.limit {
                let guild_id = self.interaction.guild_id?;

                if self.manager.remove(guild_id).await.is_ok() {
                    self.interaction
                        .channel_id
                        .say(&self.http, IDLE_ALERT)
                        .await
                        .unwrap();
                }
            }
        }
        None
    }
}
