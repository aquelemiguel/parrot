// use serenity::{
//     client::Context,
//     framework::standard::{macros::command, CommandResult},
//     model::channel::Message,
// };

// use crate::{
//     strings::{NO_VOICE_CONNECTION, QUEUE_IS_EMPTY},
//     utils::create_response,
// };

// #[command]
// async fn pause(ctx: &Context,
// interaction: &mut ApplicationCommandInteraction) -> Result<(), SerenityError> {
//     let guild_id = msg.guild(&ctx.cache).await.unwrap().id;
//     let manager = songbird::get(ctx).await.unwrap().clone();

//     let call = match manager.get(guild_id) {
//         Some(call) => call,
//         None => return create_response(&ctx.http, msg, NO_VOICE_CONNECTION).await,
//     };

//     let handler = call.lock().await;
//     let queue = handler.queue();

//     if queue.is_empty() {
//         return create_response(&ctx.http, msg, QUEUE_IS_EMPTY).await;
//     }

//     if queue.pause().is_ok() {
//         return create_response(&ctx.http, msg, "Paused!").await;
//     }

//     Ok(())
// }
