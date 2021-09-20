use crate::utils::{get_human_readable_timestamp, send_simple_message};
use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
};

#[command]
async fn queue(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild(&ctx.cache).await.unwrap().id;
    let manager = songbird::get(ctx).await.expect("Could not retrieve Songbird voice client");

    if let Some(call) = manager.get(guild_id) {
        let handler = call.lock().await;
        let tracks = handler.queue().current_queue();

        msg.channel_id.send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title("Queue");

                let top_track = tracks.first().unwrap();
                e.thumbnail(top_track.metadata().thumbnail.as_ref().unwrap());

                for (i, t) in tracks.iter().enumerate() {
                    let title = t.metadata().title.as_ref().unwrap();
                    let duration = get_human_readable_timestamp(t.metadata().duration.unwrap());

                    e.field(
                        format!("[{}] {}", i + 1, title),
                        format!("Duration: `{}`\nRequested by: `{}`", duration, msg.author.name),
                        false,
                    );
                }

                e
            })
        }).await?;
    } else {
        send_simple_message(&ctx.http, msg, "I'm not connected to any voice channel!").await;
    }

    Ok(())
}
