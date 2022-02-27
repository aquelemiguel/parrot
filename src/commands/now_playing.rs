use crate::{
    errors::ParrotError,
    strings::NOTHING_IS_PLAYING,
    utils::{create_embed_response, create_now_playing_embed, create_response},
};
use serenity::{
    client::Context, model::interactions::application_command::ApplicationCommandInteraction,
};

pub async fn now_playing(
    ctx: &Context,
    interaction: &mut ApplicationCommandInteraction,
) -> Result<(), ParrotError> {
    let guild_id = interaction.guild_id.unwrap();
    let manager = songbird::get(ctx).await.unwrap();
    let call = manager.get(guild_id).unwrap();

    let handler = call.lock().await;
    let track = match handler.queue().current() {
        Some(track) => track,
        None => return create_response(&ctx.http, interaction, NOTHING_IS_PLAYING).await,
    };

    let embed = create_now_playing_embed(&track).await;
    create_embed_response(&ctx.http, interaction, embed).await
}
