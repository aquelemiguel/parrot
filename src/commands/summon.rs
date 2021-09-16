use serenity::prelude::Mentionable;
use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
};

use crate::util::create_default_embed;

#[command]
async fn summon(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();

    let channel_id = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let connect_to = match channel_id {
        Some(channel) => {
            msg.channel_id
                .send_message(&ctx.http, |m| {
                    m.embed(|e| {
                        create_default_embed(
                            e,
                            "Summon",
                            &format!("Joining **{}**!", channel.mention()),
                        );
                        e
                    })
                })
                .await?;

            channel
        }
        None => {
            msg.channel_id
                .send_message(&ctx.http, |m| {
                    m.embed(|e| {
                        create_default_embed(
                            e,
                            "Summon",
                            "Could not find you in any voice channel!",
                        );
                        e
                    })
                })
                .await?;

            return Ok(());
        }
    };

    let manager = songbird::get(ctx).await.expect("").clone();
    let _res = manager.join(guild.id, connect_to).await;

    Ok(())
}
