use serenity::prelude::Mentionable;
use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
};

use crate::strings::{AUTHOR_NOT_DJ, AUTHOR_NOT_FOUND};
use crate::utils::{author_is_dj, send_simple_message};

#[command]
async fn summon(ctx: &Context, msg: &Message) -> CommandResult {
    if !author_is_dj(ctx, msg).await {
        send_simple_message(&ctx.http, msg, AUTHOR_NOT_DJ).await;
        return Ok(());
    } else {
        let guild = msg.guild(&ctx.cache).await.unwrap();

        // Find the voice channel where the author is at
        let channel_opt = guild
            .voice_states
            .get(&msg.author.id)
            .and_then(|voice_state| voice_state.channel_id);

        if let Some(channel_id) = channel_opt {
            send_simple_message(
                &ctx.http,
                msg,
                &format!("Joining **{}**!", channel_id.mention()),
            )
            .await;

            let manager = songbird::get(ctx)
                .await
                .expect("Could not retrieve Songbird voice client");
            manager.join(guild.id, channel_id).await.0;
        } else {
            send_simple_message(&ctx.http, msg, AUTHOR_NOT_FOUND).await;
        }
    }
    Ok(())
}
