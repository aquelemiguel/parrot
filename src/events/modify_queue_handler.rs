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
    pub ctx_data: Arc<RwLock<TypeMap>>,
    pub call: Arc<Mutex<Call>>,
    pub guild_id: GuildId,
}

#[async_trait]
impl EventHandler for ModifyQueueHandler {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(_) = ctx {
            update_queue_messages(&self.http, &self.ctx_data, &self.call, self.guild_id).await;
        };

        None
    }
}

pub async fn update_queue_messages(
    http: &Arc<Http>,
    ctx_data: &Arc<RwLock<TypeMap>>,
    call: &Arc<Mutex<Call>>,
    guild_id: GuildId,
) {
    let data = ctx_data.read().await;
    let gqi_map = data.get::<GuildQueueInteractions>().unwrap();

    let mut messages = match gqi_map.get(&guild_id) {
        Some(messages) => messages.clone(),
        None => return,
    };
    drop(data);

    for (message, page_lock) in messages.iter_mut() {
        let author = get_full_username(&message.author);

        let handler = call.lock().await;
        let tracks = handler.queue().current_queue();
        drop(handler);

        // has the page size shrunk?
        let num_pages = calculate_num_pages(&tracks);

        let mut page = page_lock.write().await;
        *page = usize::min(*page, num_pages);

        let embed = create_queue_embed(&author, &tracks, *page);

        if let Err(_) = message.edit(&http, |edit| edit.set_embed(embed)).await {
            let mut data = ctx_data.write().await;
            let gqi_map = data.get_mut::<GuildQueueInteractions>().unwrap();

            let msgs = gqi_map.get_mut(&guild_id).unwrap();
            msgs.retain(|(m, _)| m.id != message.id);
            println!("im out");
        };
    }
}
