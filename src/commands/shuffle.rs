use rand::Rng;

use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};

use crate::{strings::NO_VOICE_CONNECTION, utils::send_simple_message};

#[command]
async fn shuffle(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let guild_id = msg.guild(&ctx.cache).await.unwrap().id;
    let manager = songbird::get(ctx)
        .await
        .expect("Could not retrieve Songbird voice client");

    if let Some(call) = manager.get(guild_id) {
        let handler = call.lock().await;
        handler.queue().modify_queue(|queue| {
            // Skip the first track on queue, it's currently being played
            fisher_yates(
                queue.make_contiguous()[1..].as_mut(),
                &mut rand::thread_rng(),
            )
        });
        drop(handler);
        send_simple_message(&ctx.http, msg, "Shuffled successfully").await;
    } else {
        send_simple_message(&ctx.http, msg, NO_VOICE_CONNECTION).await;
    }

    Ok(())
}

fn fisher_yates<T, R>(values: &mut [T], mut rng: R)
where
    R: rand::RngCore + Sized,
{
    let mut index = values.len();
    while index >= 2 {
        index -= 1;
        values.swap(index, rng.gen_range(0..(index + 1)));
    }
}
