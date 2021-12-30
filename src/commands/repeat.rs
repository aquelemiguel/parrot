use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
};

use crate::{strings::NO_VOICE_CONNECTION, utils::send_simple_message};

#[command("loop")]
async fn repeat(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild(&ctx.cache).await.unwrap().id;
    let manager = songbird::get(ctx).await.unwrap();

    let call = match manager.get(guild_id) {
        Some(call) => call,
        None => return send_simple_message(&ctx.http, msg, NO_VOICE_CONNECTION).await,
    };

    let handler = call.lock().await;
    let track = handler.queue().current().unwrap();

    if track.disable_loop().is_ok() {
        return send_simple_message(&ctx.http, msg, "Disabled loop!").await;
    } else if track.enable_loop().is_ok() {
        return send_simple_message(&ctx.http, msg, "Enabled loop!").await;
    }

    Ok(())
}
