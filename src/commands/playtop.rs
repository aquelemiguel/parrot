use crate::commands::{play::execute_play, PlayFlag};

use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};

#[command]
#[aliases("pt")]
async fn playtop(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    execute_play(ctx, msg, args, &PlayFlag::PLAYTOP).await?;

    // // Handle empty requests
    // let url = match args.single::<String>() {
    //     Ok(url) => url,
    //     Err(_) => {
    //         send_simple_message(&ctx.http, msg, MISSING_PLAY_QUERY).await;
    //         return Ok(());
    //     }
    // };

    // let guild = msg.guild(&ctx.cache).await.unwrap();
    // let manager = songbird::get(ctx)
    //     .await
    //     .expect("Could not retrieve Songbird voice client");

    // // Try to join a voice channel if not in one just yet
    // summon(&ctx, &msg, args.clone()).await?;

    // //These are needed to place playlist songs at the top of queue if url is playlist
    // let mut is_playlist = false;
    // let mut num_of_songs = 0;

    // if let Some(call) = manager.get(guild.id) {
    //     // Handle an URL
    //     if url.clone().starts_with("http") {
    //         // If is a playlist
    //         if url.clone().contains("youtube.com/playlist?list=") {
    //             match YoutubeDl::new(url).flat_playlist(true).run() {
    //                 Ok(result) => {
    //                     if let YoutubeDlOutput::Playlist(playlist) = result {
    //                         let entries = playlist.entries.unwrap();
    //                         is_playlist = true;
    //                         num_of_songs = entries.len();

    //                         for entry in entries {
    //                             let uri = format!(
    //                                 "https://www.youtube.com/watch?v={}",
    //                                 entry.url.unwrap()
    //                             );
    //                             let source = Restartable::ytdl(uri, true).await?;
    //                             let mut handler = call.lock().await;
    //                             handler.enqueue_source(source.into());
    //                         }
    //                     }
    //                 }
    //                 Err(_) => todo!("Show failed to fetch playlist message!"),
    //             }
    //         }
    //         // Just a single song
    //         else {
    //             let source = Restartable::ytdl(url, true).await?;
    //             let mut handler = call.lock().await;
    //             handler.enqueue_source(source.into());
    //         }
    //     }
    //     // Play via search
    //     else {
    //         let query = args.rewind().remains().unwrap(); // Rewind and fetch the entire query
    //         let source = Restartable::ytdl_search(query, false).await?;
    //         let mut handler = call.lock().await;
    //         handler.enqueue_source(source.into());
    //     }

    //     let handler = call.lock().await;
    //     let queue = handler.queue().current_queue();
    //     drop(handler);

    //     // If it's not going to be played immediately, notify it has been enqueued
    //     if queue.len() > 1 {
    //         // Reorders the queue if needed
    //         let handler = call.lock().await;
    //         reorder_queue(&handler, is_playlist, num_of_songs);

    //         // We refetch queue to get latest changes
    //         let queue = handler.queue().current_queue();
    //         drop(handler);

    //         let top_track = &queue[1];
    //         let metadata = top_track.metadata().clone();
    //         let position = top_track.get_info().await?.position;

    //         msg.channel_id
    //             .send_message(&ctx.http, |m| {
    //                 m.embed(|e| {
    //                     e.title("Added to top of queue");
    //                     e.thumbnail(metadata.thumbnail.unwrap());

    //                     e.description(format!(
    //                         "[**{}**]({})",
    //                         metadata.title.unwrap(),
    //                         metadata.source_url.unwrap()
    //                     ));

    //                     let mut estimated_time = queue
    //                         .into_iter()
    //                         .map(|track| track.metadata().duration.unwrap())
    //                         .sum();

    //                     estimated_time -= position;

    //                     let footer_text = format!(
    //                         "Track duration: {}\nEstimated time until play: {}",
    //                         get_human_readable_timestamp(metadata.duration.unwrap()),
    //                         get_human_readable_timestamp(estimated_time)
    //                     );

    //                     let mut footer = CreateEmbedFooter::default();
    //                     footer.text(footer_text);

    //                     e.set_footer(footer)
    //                 })
    //             })
    //             .await?;
    //     } else {
    //         now_playing(&ctx, &msg, args.clone()).await?;
    //     }
    // } else {
    //     send_simple_message(&ctx.http, msg, NO_VOICE_CONNECTION).await;
    // }

    Ok(())
}

// fn reorder_queue(handler: &MutexGuard<Call>, is_playlist: bool, num_of_songs: usize) {
//     // Check if we need to move new item to top
//     if handler.queue().len() > 2 {
//         handler.queue().modify_queue(|queue| {
//             let mut non_playing = queue.split_off(1);
//             if !is_playlist {
//                 // Rotate the vec to place last added song to the front and maintain order of songs
//                 non_playing.rotate_right(1);
//             } else {
//                 // We subtract num of songs from temp length so that the first song of playlist is first
//                 let rotate_num = non_playing.len() - num_of_songs;
//                 non_playing.rotate_left(rotate_num);
//             }
//             // Append the new order to current queue which is just the current playing song
//             queue.append(&mut non_playing);
//         });
//     }
// }
