use serenity::{client::Context, framework::standard::{macros::command, CommandResult}, model::channel::Message};

#[command]
async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild(&ctx.cache).await.unwrap().id;
    let manager = songbird::get(ctx).await.expect("Could not retrieve Songbird voice client");
    
    if let Some(lock) = manager.get(guild_id) {
        let mut handler = lock.lock().await;
        handler.leave().await.expect("Failed to leave voice channel");

        msg.channel_id.send_message(&ctx.http, |m| {
            m.embed(|e| e.description("See you soon!"))
        }).await?;
    } else {
        msg.channel_id.send_message(&ctx.http, |m| {
            m.embed(|e| e.description("Not connected to any voice channel!"))
        }).await?;
    }

    Ok(())
}
