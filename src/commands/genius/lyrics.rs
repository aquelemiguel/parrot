use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};

use crate::{
    commands::genius::{genius_lyrics, genius_search},
    strings::MISSING_PLAY_QUERY,
    utils::send_simple_message,
};

#[command]
async fn lyrics(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    match args.remains() {
        Some(query) => {
            if let Some(hits) = genius_search(query).await {
                if hits.is_empty() {
                    send_simple_message(&ctx.http, &msg, "Could not fetch lyrics!").await;
                    return Ok(());
                }

                let url = hits[0]["result"]["url"].as_str().unwrap();

                match genius_lyrics(url).await {
                    Ok(lyrics) => send_lyrics_message(ctx, msg, &lyrics).await,
                    Err(_) => send_simple_message(&ctx.http, &msg, "Could not fetch lyrics!").await,
                }
            } else {
                send_simple_message(
                    &ctx.http,
                    &msg,
                    &format!("Could not find any songs that match `{}`", query),
                )
                .await;
            }
        }
        None => {
            send_simple_message(&ctx.http, &msg, MISSING_PLAY_QUERY).await;
        }
    };

    Ok(())
}

async fn send_lyrics_message(ctx: &Context, msg: &Message, lyrics: &String) {
    msg.channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                // e.title(format!("Lyrics for {}", "TODO!!!"));
                // e.url();
                // e.thumbnail();
                println!("{}", lyrics);
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
