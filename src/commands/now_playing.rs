use crate::{
    strings::{NO_VOICE_CONNECTION, QUEUE_IS_EMPTY},
    utils::{create_embed_response, create_now_playing_embed, create_response},
};
use serenity::{
    client::Context, model::interactions::application_command::ApplicationCommandInteraction,
    prelude::SerenityError,
};

pub async fn now_playing(
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

    let track = match handler.queue().current() {
        Some(track) => track,
        None => return create_response(&ctx.http, interaction, QUEUE_IS_EMPTY).await,
    };

    let embed = create_now_playing_embed(&track).await;

    create_embed_response(&ctx.http, interaction, embed).await
}
