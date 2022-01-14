// use serenity::{
//     client::Context,
//     framework::standard::{macros::command, CommandResult},
//     model::channel::Message,
// };
// use songbird::tracks::{LoopState, TrackHandle};

// use crate::{strings::NO_VOICE_CONNECTION, utils::send_simple_message};

// #[command("loop")]
// async fn repeat(ctx: &Context, msg: &Message) -> CommandResult {
//     let guild_id = msg.guild(&ctx.cache).await.unwrap().id;
//     let manager = songbird::get(ctx).await.unwrap();

//     let call = match manager.get(guild_id) {
//         Some(call) => call,
//         None => return send_simple_message(&ctx.http, msg, NO_VOICE_CONNECTION).await,
//     };

//     let handler = call.lock().await;
//     let track = handler.queue().current().unwrap();

//     let was_looping = track.get_info().await?.loops == LoopState::Infinite;
//     let toggler = if was_looping {
//         TrackHandle::disable_loop
//     } else {
//         TrackHandle::enable_loop
//     };

//     match toggler(&track) {
//         Ok(_) if was_looping => send_simple_message(&ctx.http, msg, "Disabled loop!").await,
//         Ok(_) if !was_looping => send_simple_message(&ctx.http, msg, "Enabled loop!").await,
//         _ => send_simple_message(&ctx.http, msg, "Failed to toggle loop!").await,
//     }
// }
