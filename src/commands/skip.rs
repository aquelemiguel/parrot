use crate::{
    strings::{NOTHING_IS_PLAYING, SKIPPED},
    utils::create_response,
};
use serenity::{
    client::Context, model::interactions::application_command::ApplicationCommandInteraction,
    prelude::SerenityError,
};

pub async fn skip(
    ctx: &Context,
    interaction: &mut ApplicationCommandInteraction,
) -> Result<(), SerenityError> {
    let guild_id = interaction.guild_id.unwrap();
    let manager = songbird::get(ctx).await.unwrap();
    let call = manager.get(guild_id).unwrap();

    let args = interaction.data.options.clone();
    let to_skip = match args.first() {
        Some(arg) => arg.value.as_ref().unwrap().as_u64().unwrap() as usize,
        None => 1,
    };

    let handler = call.lock().await;
    let queue = handler.queue();

    if queue.is_empty() {
        return create_response(&ctx.http, interaction, NOTHING_IS_PLAYING).await;
    } else {
        for _ in 1..to_skip {
            queue.dequeue(1);
        }
        if queue.skip().is_ok() {
            return create_response(&ctx.http, interaction, SKIPPED).await;
        }
    }

    Ok(())
}
