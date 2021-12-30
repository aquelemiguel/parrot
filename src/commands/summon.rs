use serenity::prelude::Mentionable;
use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
};

use crate::strings::AUTHOR_NOT_FOUND;
use crate::utils::send_simple_message;

#[command]
pub async fn summon(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let manager = songbird::get(ctx).await.unwrap();

    let channel_opt = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let channel_id = match channel_opt {
        Some(channel_id) => channel_id,
        None => return send_simple_message(&ctx.http, msg, AUTHOR_NOT_FOUND).await 
    };

    if let Some(call) = manager.get(guild.id) {
        let handler = call.lock().await;
        let has_current_connection = handler.current_connection().is_some();
        drop(handler);

        // bot is already in the channel
        if has_current_connection {
            return Ok(());
        }

        // bot might have been disconnected manually
        manager.remove(guild.id).await.unwrap();
    }

    // join the channel
    manager.join(guild.id, channel_id).await.1?;

    return send_simple_message(
        &ctx.http,
        msg,
        &format!("Joining **{}**!", channel_id.mention()),
    )
    .await;
}
