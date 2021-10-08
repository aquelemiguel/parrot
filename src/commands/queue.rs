use crate::{strings::NO_VOICE_CONNECTION, utils::{get_human_readable_timestamp, send_simple_message}};
use serenity::{builder::CreateEmbedFooter, client::Context, framework::standard::{macros::command, CommandResult}, model::channel::Message};

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
                let metadata = top_track.metadata();

                // e.thumbnail(top_track.metadata().thumbnail.as_ref().unwrap());

                let description = format!(
                    "[{}]({}) • `{}`",
                    metadata.title.as_ref().unwrap(),
                    metadata.source_url.as_ref().unwrap(),
                    get_human_readable_timestamp(metadata.duration.unwrap())
                );

                e.field("🔊  Now playing", description, false);

                let mut description = String::new();

                for (i, t) in tracks.iter().skip(1).enumerate() {
                    let title = t.metadata().title.as_ref().unwrap();
                    let url = t.metadata().source_url.as_ref().unwrap();
                    let duration = get_human_readable_timestamp(t.metadata().duration.unwrap());

                    description.push_str(&format!("`{}.` [{}]({}) • `{}`\n", i+1, title, url, duration));
                }

                e.field("⏬  Up next", description, false)
            })
        }).await?;
    } else {
        send_simple_message(&ctx.http, msg, NO_VOICE_CONNECTION).await;
    }

    Ok(())
}
