use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
};

#[command]
async fn stop(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild(&ctx.cache).await.unwrap().id;
    let manager = songbird::get(ctx).await.expect("Could not retrieve Songbird voice client");

    if let Some(call) = manager.get(guild_id) {
        let handler = call.lock().await;
        let queue = handler.queue();

        if queue.is_empty() {
            msg.channel_id.send_message(&ctx.http, |m| {
                m.embed(|e| e.description("The queue is already empty!"))
            }).await?;
        } else {
            queue.stop();

            msg.channel_id.send_message(&ctx.http, |m| {
                m.embed(|e| e.description("Stopped!"))
            }).await?;
        }
    } else {
        msg.channel_id.send_message(&ctx.http, |m| {
            m.embed(|e| e.description("I'm not connected to any voice channel!"))
        }).await?;
    }

    Ok(())
}
