use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
};

use crate::{strings::NO_VOICE_CONNECTION, utils::send_simple_message};

#[command]
async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild(&ctx.cache).await.unwrap().id;
    let manager = songbird::get(ctx)
        .await
        .expect("Could not retrieve Songbird voice client");

    if let Some(lock) = manager.get(guild_id) {
        let mut handler = lock.lock().await;
        handler
            .leave()
            .await
            .expect("Failed to leave voice channel");
        send_simple_message(&ctx.http, msg, "See you soon!").await;
    } else {
        send_simple_message(&ctx.http, msg, NO_VOICE_CONNECTION).await;
    }

    Ok(())
}
