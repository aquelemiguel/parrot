// use serenity::{
//     client::Context,
//     framework::standard::{macros::command, CommandResult},
//     model::channel::Message,
// };

// use crate::{strings::NO_VOICE_CONNECTION, utils::create_response};

// #[command]
// #[aliases("disconnect", "dc", "exit")]
// async fn leave(ctx: &Context,
// interaction: &mut ApplicationCommandInteraction) -> Result<(), SerenityError> {
//     let guild_id = msg.guild(&ctx.cache).await.unwrap().id;
//     let manager = songbird::get(ctx).await.unwrap();

//     if manager.get(guild_id).is_some() {
//         manager.remove(guild_id).await?;
//         create_response(&ctx.http, msg, "See you soon!").await
//     } else {
//         create_response(&ctx.http, msg, NO_VOICE_CONNECTION).await
//     }
// }
