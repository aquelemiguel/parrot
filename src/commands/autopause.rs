use std::{collections::HashMap, sync::Arc};

use serenity::{
    async_trait,
    client::Context,
    model::{id::GuildId, interactions::application_command::ApplicationCommandInteraction},
    prelude::{Mutex, RwLock, SerenityError, TypeMap, TypeMapKey},
};
use songbird::{Call, Event, EventContext, EventHandler, TrackEvent};

use crate::{strings::NO_VOICE_CONNECTION, utils::create_response};

#[derive(Default)]
struct GuildSettings {
    autopause: bool,
}

struct GuildSettingsMap;
impl TypeMapKey for GuildSettingsMap {
    type Value = HashMap<GuildId, GuildSettings>;
}

struct SongEndNotifier {
    guild_id: GuildId,
    call: Arc<Mutex<Call>>,
    ctx_data: Arc<RwLock<TypeMap>>,
}

#[async_trait]
impl EventHandler for SongEndNotifier {
    async fn act(&self, _ctx: &EventContext<'_>) -> Option<Event> {
        let reader = self.ctx_data.read().await;
        let settings = reader.get::<GuildSettingsMap>().unwrap();
        let guild_settings = match settings.get(&self.guild_id) {
            Some(guild_settings) => guild_settings,
            _ => return None,
        };

        println!("Autopause value is {}", guild_settings.autopause);
        if guild_settings.autopause {
            println!("Pausing...");
            let handler = self.call.lock().await;
            let queue = handler.queue();
            queue.pause().ok();
        }
        None
    }
}

pub async fn autopause(
    ctx: &Context,
    interaction: &mut ApplicationCommandInteraction,
) -> Result<(), SerenityError> {
    let guild_id = interaction.guild_id.unwrap();
    let manager = songbird::get(ctx).await.unwrap();

    let call = match manager.get(guild_id) {
        Some(call) => call,
        None => return create_response(&ctx.http, interaction, NO_VOICE_CONNECTION).await,
    };

    let mut data = ctx.data.write().await;
    let settings = data.get_mut::<GuildSettingsMap>().unwrap();
    let guild_settings = settings.entry(guild_id).or_default();

    create_response(
        &ctx.http,
        interaction,
        &format!("Autopause value was {}", guild_settings.autopause),
    )
    .await
    .unwrap();
    guild_settings.autopause = !guild_settings.autopause;
    create_response(
        &ctx.http,
        interaction,
        &format!("Autopause value is now {}", guild_settings.autopause),
    )
    .await
    .unwrap();

    let handler = call.lock().await;
    let queue = handler.queue().current_queue();

    queue
        .first()
        .unwrap()
        .add_event(
            Event::Track(TrackEvent::End),
            SongEndNotifier {
                guild_id,
                call: call.clone(),
                ctx_data: ctx.data.clone(),
            },
        )
        .unwrap();

    Ok(())
}
