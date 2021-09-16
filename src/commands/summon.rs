use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use std::time::Duration;

use serenity::prelude::Mentionable;
use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
};
use songbird::Event;

use crate::events::idle_notifier::IdleNotifier;
use crate::util::create_default_embed;

#[command]
async fn summon(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();

    let channel_id = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let connect_to = match channel_id {
        Some(channel) => {
            msg.channel_id
                .send_message(&ctx.http, |m| {
                    m.embed(|e| {
                        create_default_embed(
                            e,
                            "Summon",
                            &format!("Joining **{}**!", channel.mention()),
                        );
                        e
                    })
                })
                .await?;

            channel
        }
        None => {
            msg.channel_id
                .send_message(&ctx.http, |m| {
                    m.embed(|e| {
                        create_default_embed(
                            e,
                            "Summon",
                            "Could not find you in any voice channel!",
                        );
                        e
                    })
                })
                .await?;

            return Ok(());
        }
    };

    let manager = songbird::get(ctx).await.expect("").clone();
    let lock = manager.join(guild.id, connect_to).await.0;

    let mut handler = lock.lock().await;
    handler.add_global_event(
        Event::Periodic(Duration::from_secs(60), None),
        IdleNotifier {
            message: msg.clone(),
            manager,
            count: Arc::new(AtomicUsize::new(1)),
            http: ctx.http.clone(),
        },
    );

    Ok(())
}
