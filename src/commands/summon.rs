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
use crate::strings::AUTHOR_NOT_FOUND;
use crate::utils::send_simple_message;

#[command]
async fn summon(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();

    // Find the voice channel where the author is at
    let channel_opt = guild.voice_states.get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    if let Some(channel_id) = channel_opt {
        send_simple_message(&ctx.http, msg, &format!("Joining **{}**!", channel_id.mention())).await;

        let manager = songbird::get(ctx).await.expect("Could not retrieve Songbird voice client");
        let call = manager.join(guild.id, channel_id).await.0;
        let mut handler = call.lock().await;

        let action = IdleNotifier {
            message: msg.clone(),
            manager,
            count: Arc::new(AtomicUsize::new(1)),
            http: ctx.http.clone()
        };

        handler.add_global_event(Event::Periodic(Duration::from_secs(1), None), action);
    }
    else {
        send_simple_message(&ctx.http, msg, AUTHOR_NOT_FOUND).await;
    }

    Ok(())
}
