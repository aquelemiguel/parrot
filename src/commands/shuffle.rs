use crate::{
    errors::ParrotError, handlers::track_end::update_queue_messages, messaging::Response,
    utils::create_response_,
};
use rand::Rng;
use serenity::{
    client::Context, model::interactions::application_command::ApplicationCommandInteraction,
};

pub async fn shuffle(
    ctx: &Context,
    interaction: &mut ApplicationCommandInteraction,
) -> Result<(), ParrotError> {
    let guild_id = interaction.guild_id.unwrap();
    let manager = songbird::get(ctx).await.unwrap();
    let call = manager.get(guild_id).unwrap();

    let handler = call.lock().await;
    handler.queue().modify_queue(|queue| {
        // skip the first track on queue because it's being played
        fisher_yates(
            queue.make_contiguous()[1..].as_mut(),
            &mut rand::thread_rng(),
        )
    });

    // refetch the queue after modification
    let queue = handler.queue().current_queue();
    drop(handler);

    create_response_(&ctx.http, interaction, Response::Shuffled).await?;
    update_queue_messages(&ctx.http, &ctx.data, &queue, guild_id).await;
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
