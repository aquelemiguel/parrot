use crate::{
    commands::genius::get_genius_song_id,
    commands::genius::send_genius_request,
    strings::{MISSING_PLAY_QUERY, NO_VOICE_CONNECTION},
    utils::send_simple_message,
};

use html2md::parse_html;
use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};

struct GeniusExplanation {
    text: String,
    thumbnail: String,
    page_url: String,
    song: String,
}

#[command]
async fn explain(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    match args.remains() {
        Some(query) => {
            let song_id = get_genius_song_id(query).await.unwrap();
            let explanation = get_genius_song_explanation(song_id).await;
            send_explanation_message(ctx, msg, explanation).await;
        }
        None => {
            let guild_id = msg.guild(&ctx.cache).await.unwrap().id;
            let manager = songbird::get(ctx)
                .await
                .expect("Could not retrieve Songbird voice client");

            if let Some(call) = manager.get(guild_id) {
                let handler = call.lock().await;

                match handler.queue().current_queue().first() {
                    Some(track) => {
                        let stream_title = track.metadata().title.as_ref().unwrap();
                        let song_id = get_genius_song_id(&stream_title).await.unwrap();
                        let explanation = get_genius_song_explanation(song_id).await;
                        send_explanation_message(ctx, msg, explanation).await;
                    }
                    None => {
                        send_simple_message(&ctx.http, msg, MISSING_PLAY_QUERY).await;
                    }
                };
            } else {
                send_simple_message(&ctx.http, msg, NO_VOICE_CONNECTION).await;
            }
        }
    };

    Ok(())
}

async fn send_explanation_message(ctx: &Context, msg: &Message, explanation: GeniusExplanation) {
    msg.channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title(format!("Explaining {}", explanation.song));
                e.url(explanation.page_url);
                e.thumbnail(explanation.thumbnail);
                e.description(explanation.text);

                e.footer(|f| {
                    f.text("Powered by Genius");
                    f.icon_url("https://bit.ly/3BOic6A")
                })
            })
        })
        .await
        .unwrap();
}

async fn get_genius_song_explanation(id: i64) -> GeniusExplanation {
    let res = send_genius_request(format!("songs/{}?text_format=html", id))
        .await
        .unwrap();

    let song_title = res["response"]["song"]["title"]
        .as_str()
        .unwrap()
        .to_string();

    let artist = res["response"]["song"]["primary_artist"]["name"]
        .as_str()
        .unwrap()
        .to_string();

    let thumbnail = res["response"]["song"]["song_art_image_url"]
        .as_str()
        .unwrap()
        .to_string();

    let page_url = res["response"]["song"]["url"].as_str().unwrap().to_string();

    let text = parse_html(
        res["response"]["song"]["description"]["html"]
            .as_str()
            .unwrap(),
    );

    GeniusExplanation {
        text,
        thumbnail,
        page_url,
        song: format!("\"{}\" by {}", song_title, artist),
    }
}
