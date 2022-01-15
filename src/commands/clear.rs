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
// #[aliases("cl")]
// async fn clear(
//     ctx: &Context,
//     interaction: &mut ApplicationCommandInteraction,
// ) -> Result<(), SerenityError> {
//     let guild_id = msg.guild(&ctx.cache).await.unwrap().id;
//     let manager = songbird::get(ctx).await.unwrap();

//     let call = match manager.get(guild_id) {
//         Some(call) => call,
//         None => return create_response(&ctx.http, msg, NO_VOICE_CONNECTION).await,
//     };

//     let handler = call.lock().await;
//     let queue = handler.queue().current_queue();

//     if queue.is_empty() {
//         return create_response(&ctx.http, msg, QUEUE_IS_EMPTY).await;
//     }

//     handler.queue().modify_queue(|v| {
//         v.drain(1..);
//     });

//     create_response(&ctx.http, msg, "Cleared!").await
// }
