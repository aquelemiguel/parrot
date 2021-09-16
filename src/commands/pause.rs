use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
};

use crate::util::create_default_embed;

#[command]
async fn pause(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let manager = songbird::get(ctx).await.expect("").clone();

    if let Some(lock) = manager.get(guild.id) {
        let handler = lock.lock().await;

        if handler.queue().is_empty() {
            msg.channel_id
                .send_message(&ctx.http, |m| {
                    m.embed(|e| {
                        create_default_embed(e, "Pause", "There is nothing playing!");
                        e
                    })
                })
                .await?;
        } else if handler.queue().pause().is_ok() {
            msg.channel_id
                .send_message(&ctx.http, |m| {
                    m.embed(|e| {
                        create_default_embed(e, "Pause", "Paused!");
                        e
                    })
                })
                .await?;
        }
    } else {
        msg.channel_id
            .send_message(&ctx.http, |m| {
                m.embed(|e| {
                    create_default_embed(e, "Pause", "I'm not connected to any voice channel!");
                    e
                })
            })
            .await?;
    }

    Ok(())
}
