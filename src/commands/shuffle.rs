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
    let manager = songbird::get(ctx).await.unwrap();

    let call = match manager.get(guild_id) {
        Some(call) => call,
        None => return send_simple_message(&ctx.http, msg, NO_VOICE_CONNECTION).await,
    };

    let handler = call.lock().await;
    handler.queue().modify_queue(|queue| {
        // skip the first track on queue because it's being played
        shuffle_values(
            queue.make_contiguous()[1..].as_mut(),
            &mut rand::thread_rng(),
        )
    });
    return send_simple_message(&ctx.http, msg, "Shuffled successfully").await;
}

fn shuffle_values<T, R>(values: &mut [T], mut rng: R)
where
    R: rand::RngCore + Sized,
{
    let mut index = values.len();
    while index >= 2 {
        index -= 1;
        values.swap(index, rng.gen_range(0..(index + 1)));
    }
}
