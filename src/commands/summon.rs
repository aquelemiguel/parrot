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

    // Find the voice channel where the author is at
    let channel_opt = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    if let Some(channel_id) = channel_opt {
        let manager = songbird::get(ctx)
            .await
            .expect("Could not retrieve Songbird voice client");

        if let Some(call) = manager.get(guild.id) {
            let handler = call.lock().await;
            let current_connection = handler.current_connection();

            // Bot might have been disconnected manually
            if current_connection.is_none() {
                drop(handler);
                manager
                    .remove(guild.id)
                    .await
                    .expect("Could not drop handler");
            } else {
                drop(handler);
                return Ok(()); // Bot is already in the channel
            }
        }

        // Now that we've ensured the bot isn't connected, join the channel
        manager.join(guild.id, channel_id).await.1?;

        send_simple_message(
            &ctx.http,
            msg,
            &format!("Joining **{}**!", channel_id.mention()),
        )
        .await;
    } else {
        send_simple_message(&ctx.http, msg, AUTHOR_NOT_FOUND).await;
    }

    Ok(())
}
