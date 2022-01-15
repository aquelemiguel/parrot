use serenity::client::Context;
use serenity::model::interactions::application_command::ApplicationCommandInteraction;
use serenity::model::interactions::InteractionResponseType;
use serenity::prelude::Mentionable;

// use crate::events::idle_handler::IdleHandler;
use crate::strings::AUTHOR_NOT_FOUND;

pub async fn summon(ctx: &Context, interaction: &mut ApplicationCommandInteraction) {
    let guild_id = interaction.guild_id.unwrap();
    let guild = ctx.cache.guild(guild_id).await.unwrap();

    let manager = songbird::get(ctx).await.unwrap();

    let channel_opt = guild
        .voice_states
        .get(&interaction.user.id)
        .and_then(|voice_state| voice_state.channel_id);

    let channel_id = match channel_opt {
        Some(channel_id) => channel_id,
        None => {
            return interaction
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(AUTHOR_NOT_FOUND))
                })
                .await
                .unwrap();
        }
    };

    if let Some(call) = manager.get(guild.id) {
        let handler = call.lock().await;
        let has_current_connection = handler.current_connection().is_some();
        drop(handler);

        // bot is already in the channel
        if has_current_connection {
            return;
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

        // handler.add_global_event(
        //     Event::Periodic(Duration::from_secs(1), None),
        //     IdleHandler {
        //         http: ctx.http.clone(),
        //         manager,
        //         msg: msg.clone(),
        //         limit: 60 * 10,
        //         count: Default::default(),
        //     },
        // );
    }

    let content = format!("Joining **{}**!", channel_id.mention());

    interaction
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| message.content(content))
        })
        .await
        .unwrap();
}
