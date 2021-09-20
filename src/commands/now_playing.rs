use crate::utils::{get_human_readable_timestamp, send_simple_message};
use serenity::{
    builder::CreateEmbedFooter,
    client::Context,
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
};

#[command]
#[aliases("np")]
async fn now_playing(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild(&ctx.cache).await.unwrap().id;
    let manager = songbird::get(ctx).await.expect("Could not retrieve Songbird voice client");

    if let Some(call) = manager.get(guild_id) {
        let handler = call.lock().await;

        if let Some(track) = handler.queue().current() {
            let position = track.get_info().await?.position;
            let duration = track.metadata().duration.unwrap();
            let thumbnail = track.metadata().thumbnail.as_ref().unwrap();

            msg.channel_id.send_message(&ctx.http, |m| {
                m.embed(|e| {
                    e.title("Now playing");
                    e.thumbnail(thumbnail);

                    let title = track.metadata().title.as_ref().unwrap();
                    let url = track.metadata().source_url.as_ref().unwrap();
                    e.description(format!("[**{}**]({})", title, url));

                    let mut footer = CreateEmbedFooter::default();
                    let position_human = get_human_readable_timestamp(position);
                    let duration_human = get_human_readable_timestamp(duration);

                    footer.text(format!("{} / {}", position_human, duration_human));
                    e.set_footer(footer)
                })
            }).await?;
        } else {
            send_simple_message(&ctx.http, msg, "The queue is empty!").await;
        }
    } else {
        send_simple_message(&ctx.http, msg, "I'm not connected to any voice channel!").await;
    }

    Ok(())
}
