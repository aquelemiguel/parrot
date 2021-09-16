use crate::util::{create_default_embed, get_human_readable_timestamp};
use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
};

#[command]
async fn queue(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let manager = songbird::get(ctx).await.expect("").clone();

    if let Some(lock) = manager.get(guild.id) {
        let handler = lock.lock().await;
        let tracks = handler.queue().current_queue();

        msg.channel_id
            .send_message(&ctx.http, |m| {
                m.embed(|e| {
                    e.title("Queue");

                    let top_track = tracks.first().unwrap();
                    e.thumbnail(top_track.metadata().thumbnail.as_ref().unwrap());

                    for (i, t) in tracks.iter().enumerate() {
                        let title = t.metadata().title.as_ref().unwrap();
                        let duration = get_human_readable_timestamp(t.metadata().duration.unwrap());

                        e.field(
                            format!("[{}] {}", i + 1, title),
                            format!(
                                "Duration: `{}`\nRequested by: `{}`",
                                duration, msg.author.name
                            ),
                            false,
                        );
                    }

                    e
                })
            })
            .await?;
    } else {
        msg.channel_id
            .send_message(&ctx.http, |m| {
                m.embed(|e| {
                    create_default_embed(e, "Queue", "Not in a voice channel!");
                    e
                })
            })
            .await?;
    }

    Ok(())
}
