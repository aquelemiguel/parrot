use crate::{
    commands::genius::genius_search, commands::genius::GeniusExplanation,
    strings::MISSING_PLAY_QUERY, utils::send_simple_message,
};

use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};

use super::genius_description;

#[command]
async fn explain(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    match args.remains() {
        Some(query) => {
            if let Some(hits) = genius_search(query).await {
                if hits.is_empty() {
                    send_simple_message(&ctx.http, &msg, "Could not fetch explanation!").await;
                    return Ok(());
                }

                let id = hits[0]["result"]["id"].as_i64().unwrap();

                match genius_description(id).await {
                    Ok(explanation) => send_explanation_message(ctx, msg, explanation).await,
                    Err(_) => {
                        send_simple_message(&ctx.http, &msg, "Could not fetch explanation!").await
                    }
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
