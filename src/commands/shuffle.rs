use crate::events::modify_queue_handler::update_queue_messages;
use crate::strings::NO_VOICE_CONNECTION;
use crate::utils::create_response;
use rand::Rng;
use serenity::client::Context;
use serenity::model::interactions::application_command::ApplicationCommandInteraction;
use serenity::prelude::SerenityError;

pub async fn shuffle(
    ctx: &Context,
    interaction: &mut ApplicationCommandInteraction,
) -> Result<(), SerenityError> {
    let guild_id = interaction.guild_id.unwrap();
    let manager = songbird::get(ctx).await.unwrap();

    let call = match manager.get(guild_id) {
        Some(call) => call,
        None => return create_response(&ctx.http, interaction, NO_VOICE_CONNECTION).await,
    };

    let handler = call.lock().await;
    handler.queue().modify_queue(|queue| {
        // skip the first track on queue because it's being played
        fisher_yates(
            queue.make_contiguous()[1..].as_mut(),
            &mut rand::thread_rng(),
        )
    });

    drop(handler);
    update_queue_messages(&ctx.http, &ctx.data, &call, guild_id).await;

    create_response(&ctx.http, interaction, "Shuffled successfully!").await
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
