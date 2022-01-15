// use crate::{
//     commands::genius::genius_description, commands::genius::genius_search,
//     commands::genius::genius_song, strings::MISSING_QUERY, utils::create_response,
// };

// use serde_json::Value;
// use serenity::{
//     client::Context,
//     framework::standard::{macros::command, Args, CommandResult},
//     model::channel::Message,
// };

// #[command]
// async fn explain(ctx: &Context,
// interaction: &mut ApplicationCommandInteraction, args: Args) -> Result<(), SerenityError> {
//     let query = match args.remains() {
//         Some(query) => query,
//         None => return create_response(&ctx.http, msg, MISSING_QUERY).await,
//     };

//     let hits = match genius_search(query).await {
//         Some(hits) if !hits.is_empty() => hits,
//         _ => {
//             return create_response(
//                 &ctx.http,
//                 msg,
//                 &format!("Could not find any songs that match `{}`", query),
//             )
//             .await
//         }
//     };

//     let id = hits[0]["result"]["id"].as_i64().unwrap();
//     let song = genius_song(id).await.unwrap();
//     match genius_description(&song).await {
//         Ok(explanation) => send_explanation_message(ctx, msg, &explanation, &song).await,
//         Err(_) => create_response(&ctx.http, msg, "Could not fetch explanation!").await,
//     }
// }

// async fn send_explanation_message(
//     ctx: &Context,
//     msg: &Message,
//     explanation: &str,
//     song: &Value,
// ) -> Result<(), SerenityError> {
//     msg.channel_id
//         .send_message(&ctx.http, |m| {
//             m.embed(|e| {
//                 let song_title = song["title"].as_str().unwrap().to_string();
//                 let artist = song["primary_artist"]["name"].as_str().unwrap().to_string();

//                 e.title(format!(
//                     "Explaining {}",
//                     format!("\"{}\" by {}", song_title, artist)
//                 ));

//                 let url = song["url"].as_str().unwrap().to_string();
//                 e.url(url);

//                 let thumbnail = song["song_art_image_url"].as_str().unwrap().to_string();
//                 e.thumbnail(thumbnail);

//                 e.description(explanation);

//                 e.footer(|f| {
//                     f.text("Powered by Genius");
//                     f.icon_url("https://bit.ly/3BOic6A")
//                 })
//             })
//         })
//         .await?;
//     Ok(())
// }
