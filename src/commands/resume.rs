use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
};

use crate::util::create_default_embed;

#[command]
async fn resume(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let manager = songbird::get(ctx).await.expect("").clone();

    if let Some(lock) = manager.get(guild.id) {
        let handler = lock.lock().await;

        if handler.queue().resume().is_ok() {
            msg.channel_id
                .send_message(&ctx.http, |m| {
                    m.embed(|e| {
                        create_default_embed(e, "Resume", "Resumed!");
                        e
                    })
                })
                .await?;
        }
    } else {
        msg.channel_id
            .send_message(&ctx.http, |m| {
                m.embed(|e| {
                    create_default_embed(e, "Resume", "Not in a voice channel!");
                    e
                })
            })
            .await?;
    }

    Ok(())
}
