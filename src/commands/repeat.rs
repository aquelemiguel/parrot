use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
};
use songbird::tracks::LoopState;

use crate::{
    strings::{AUTHOR_NOT_DJ, NO_VOICE_CONNECTION},
    utils::{author_is_dj, send_simple_message},
};

#[command("loop")]
async fn repeat(ctx: &Context, msg: &Message) -> CommandResult {
    if !author_is_dj(ctx, msg).await {
        send_simple_message(&ctx.http, msg, AUTHOR_NOT_DJ).await;
        return Ok(());
    } else {
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
            drop(handler);

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
    }
    Ok(())
}
