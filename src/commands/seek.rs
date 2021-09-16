use std::time::Duration;

use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult, Delimiter},
    model::channel::Message,
};

use crate::util::create_default_embed;

#[command]
#[num_args(1)]
async fn seek(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let manager = songbird::get(ctx).await.expect("").clone();

    if let Some(lock) = manager.get(guild.id) {
        let handler = lock.lock().await;

        let seek_time = match args.single::<String>() {
            Ok(t) => t,
            Err(_) => {
                msg.channel_id
                    .send_message(&ctx.http, |m| {
                        m.embed(|e| {
                            create_default_embed(e, "Seek", "Include a timestamp!");
                            e
                        })
                    })
                    .await?;

                return Ok(());
            }
        };

        let mut clock = Args::new(&seek_time, &[Delimiter::Single(':')]);
        let mins = clock.single::<u64>().unwrap();
        let secs = clock.single::<u64>().unwrap();

        let track = handler.queue().current().unwrap();
        track
            .seek_time(Duration::from_secs(mins * 60 + secs))
            .unwrap();

        msg.channel_id
            .send_message(&ctx.http, |m| {
                m.embed(|e| {
                    create_default_embed(
                        e,
                        "Seek",
                        &format!("Seeked current track to **{}**.", seek_time),
                    );
                    e
                })
            })
            .await?;
    }

    Ok(())
}
