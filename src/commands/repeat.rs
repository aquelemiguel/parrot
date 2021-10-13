use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
};
use songbird::tracks::LoopState;

use crate::{strings::NO_VOICE_CONNECTION, utils::send_simple_message};

#[command("loop")]
async fn repeat(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild(&ctx.cache).await.unwrap().id;
    let manager = songbird::get(ctx)
        .await
        .expect("Could not retrieve Songbird voice client");

    if let Some(call) = manager.get(guild_id) {
        let handler = call.lock().await;
        let track = handler
            .queue()
            .current()
            .expect("Failed to fetch handle for current track");

        if track.get_info().await?.loops == LoopState::Infinite {
            if track.disable_loop().is_ok() {
                send_simple_message(&ctx.http, msg, "Disabled loop!").await;
            }
        } else {
            if track.enable_loop().is_ok() {
                send_simple_message(&ctx.http, msg, "Enabled loop!").await;
            }
        }
    } else {
        send_simple_message(&ctx.http, msg, NO_VOICE_CONNECTION).await;
    }

    Ok(())
}
