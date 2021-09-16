use crate::util::create_default_embed;
use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
};
use songbird::tracks::LoopState;

#[command("loop")]
async fn repeat(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let manager = songbird::get(ctx).await.expect("").clone();

    if let Some(lock) = manager.get(guild.id) {
        let handler = lock.lock().await;
        let track = handler.queue().current().unwrap();

        if track.get_info().await?.loops == LoopState::Infinite {
            track.disable_loop().unwrap();

            msg.channel_id
                .send_message(&ctx.http, |m| {
                    m.embed(|e| {
                        create_default_embed(e, "Loop", "Disabled!");
                        e
                    })
                })
                .await?;
        } else {
            track.enable_loop().unwrap();

            msg.channel_id
                .send_message(&ctx.http, |m| {
                    m.embed(|e| {
                        create_default_embed(e, "Loop", "Enabled!");
                        e
                    })
                })
                .await?;
        }
    }

    Ok(())
}
