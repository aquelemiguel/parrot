use std::sync::Arc;

use serenity::{
    async_trait,
    http::Http,
    model::id::GuildId,
    prelude::{Mutex, RwLock, TypeMap},
};
use songbird::{Call, Event, EventContext, EventHandler};

use crate::{
    client::GuildQueueInteractions,
    commands::queue::{calculate_num_pages, create_queue_embed},
    utils::get_full_username,
};

pub struct ModifyQueueHandler {
    pub http: Arc<Http>,
    pub data: Arc<RwLock<TypeMap>>,
    pub call: Arc<Mutex<Call>>,
    pub guild_id: GuildId,
}

#[async_trait]
impl EventHandler for ModifyQueueHandler {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(_) = ctx {
            let data = self.data.read().await;
            let gqi_map = data.get::<GuildQueueInteractions>().unwrap();
            let mut messages = gqi_map.get(&self.guild_id).unwrap().clone();

            for (message, page_lock) in messages.iter_mut() {
                let author = get_full_username(&message.author);

                let handler = self.call.lock().await;
                let tracks = handler.queue().current_queue();
                drop(handler);

                // has the page size shrunk?
                let num_pages = calculate_num_pages(&tracks);

                let mut page = page_lock.write().await;
                *page = usize::min(*page, num_pages);

                let embed = create_queue_embed(&author, &tracks, *page);

                message
                    .edit(&self.http, |edit| edit.set_embed(embed))
                    .await
                    .unwrap();
            }
        };

        None
    }
}
