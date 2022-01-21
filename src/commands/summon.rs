use std::time::Duration;

use serenity::client::Context;
use serenity::model::interactions::application_command::ApplicationCommandInteraction;
use serenity::prelude::Mentionable;
use serenity::prelude::SerenityError;
use songbird::Event;
use songbird::TrackEvent;

use crate::events::idle_handler::IdleHandler;
use crate::events::track_end_handler::TrackEndHandler;
use crate::strings::AUTHOR_NOT_FOUND;
use crate::utils::create_response;

pub async fn summon(
    ctx: &Context,
    interaction: &mut ApplicationCommandInteraction,
    send_reply: bool,
) -> Result<(), SerenityError> {
    let guild_id = interaction.guild_id.unwrap();
    let guild = ctx.cache.guild(guild_id).await.unwrap();

    let manager = songbird::get(ctx).await.unwrap();

    let channel_opt = guild
        .voice_states
        .get(&interaction.user.id)
        .and_then(|voice_state| voice_state.channel_id);

    let channel_id = match channel_opt {
        Some(channel_id) => channel_id,
        None if send_reply => {
            return create_response(&ctx.http, interaction, AUTHOR_NOT_FOUND).await
        }
        _ => return Err(SerenityError::Other("Author not found")),
    };

    if let Some(call) = manager.get(guild.id) {
        let handler = call.lock().await;
        let has_current_connection = handler.current_connection().is_some();
        drop(handler);

        // bot is already in the channel
        if has_current_connection {
            if send_reply {
                return create_response(&ctx.http, interaction, "I'm already here!").await;
            }
            return Ok(());
        }

        // bot might have been disconnected manually
        manager.remove(guild.id).await.unwrap();
    }

    // join the channel
    manager.join(guild.id, channel_id).await.1.unwrap();

    // unregister existing events and register idle notifier
    if let Some(call) = manager.get(guild.id) {
        let mut handler = call.lock().await;

        handler.remove_all_global_events();

        handler.add_global_event(
            Event::Periodic(Duration::from_secs(1), None),
            IdleHandler {
                http: ctx.http.clone(),
                manager,
                interaction: interaction.clone(),
                limit: 60 * 10,
                count: Default::default(),
            },
        );

        handler.add_global_event(
            Event::Track(TrackEvent::End),
            TrackEndHandler {
                guild_id: guild.id,
                call: call.clone(),
                ctx_data: ctx.data.clone(),
            },
        );
    }

    if send_reply {
        let content = format!("Joining **{}**!", channel_id.mention());
        return create_response(&ctx.http, interaction, &content).await;
    }

    Ok(())
}
