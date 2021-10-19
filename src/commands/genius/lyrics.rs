use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};

use crate::{
    commands::genius::{genius_lyrics, genius_search},
    utils::send_simple_message,
};

#[command]
async fn lyrics(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    match args.remains() {
        Some(query) => {
            if let Some(hits) = genius_search(query).await {
                let url = hits[0]["result"]["url"].as_str().unwrap();

                match genius_lyrics(url).await {
                    Ok(lyrics) => send_lyrics_message(ctx, msg, &lyrics).await,
                    Err(_) => send_simple_message(&ctx.http, &msg, "Could not fetch lyrics!").await,
                }
            }
        }
        None => {
            // let guild_id = msg.guild(&ctx.cache).await.unwrap().id;
            // let manager = songbird::get(ctx)
            //     .await
            //     .expect("Could not retrieve Songbird voice client");

            // if let Some(call) = manager.get(guild_id) {
            //     let handler = call.lock().await;

            //     match handler.queue().current_queue().first() {
            //         Some(track) => {
            //             let stream_title = track.metadata().title.as_ref().unwrap();
            //             let song_id = get_genius_song_id(&stream_title).await.unwrap();
            //             let explanation = get_genius_song_explanation(song_id).await;
            //             send_explanation_message(ctx, msg, explanation).await;
            //         }
            //         None => {
            //             send_simple_message(&ctx.http, msg, MISSING_PLAY_QUERY).await;
            //         }
            //     };
            // } else {
            //     send_simple_message(&ctx.http, msg, NO_VOICE_CONNECTION).await;
            // }
        }
    };

    Ok(())

    // if let Some(call) = manager.get(guild_id) {
    //     let handler = call.lock().await;
    //     let queue = handler.queue().current_queue();

    //     let top_track = match queue.first() {
    //         Some(track) => track,
    //         None => {
    //             send_simple_message(&ctx.http, msg, QUEUE_IS_EMPTY).await;
    //             return Ok(());
    //         }
    //     };

    //     let query = format!(
    //         "{} {}",
    //         top_track.metadata().artist.as_ref().unwrap(),
    //         top_track.metadata().title.as_ref().unwrap()
    //     );
    //     println!("{}", query);

    //     let genius = Genius::new(env::var("GENIUS_TOKEN").unwrap());
    //     let hits = genius.search(&query).await.unwrap();

    //     genius.get_lyrics(url)

    //     let top_result = &hits[0];
    //     println!("{}", top_result.result.url);
    // } else {
    // }
}

async fn send_lyrics_message(ctx: &Context, msg: &Message, lyrics: &String) {
    msg.channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                // e.title(format!("Lyrics for {}", "TODO!!!"));
                // e.url();
                // e.thumbnail();
                e.description(lyrics);

                e.footer(|f| {
                    f.text("Powered by Genius");
                    f.icon_url("https://bit.ly/3BOic6A")
                })
            })
        })
        .await
        .unwrap();
}
