use serenity::{
    framework::{standard::macros::group, StandardFramework},
    Client,
};
use songbird::SerenityInit;
use std::env;

use parrot::{
    commands::{
        clear::*,
        genius::{explain::*, lyrics::*},
        leave::*,
        now_playing::*,
        pause::*,
        play::*,
        playtop::*,
        prefix::*,
        queue::*,
        remove::*,
        repeat::*,
        resume::*,
        seek::*,
        shuffle::*,
        skip::*,
        stop::*,
        summon::*,
        version::*,
    },
    events::serenity_handler::SerenityHandler,
    utils::get_prefixes,
};

#[group]
#[commands(
    clear,
    explain,
    leave,
    lyrics,
    now_playing,
    pause,
    play,
    playtop,
    prefix,
    queue,
    remove,
    repeat,
    resume,
    seek,
    shuffle,
    skip,
    stop,
    summon,
    version
)]
struct Commands;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let framework = StandardFramework::new()
        .configure(|c| {
            c.dynamic_prefix(|ctx, msg| {
                Box::pin(async move {
                    let prefixes = get_prefixes();
                    let default_prefix = env::var("PREFIX").unwrap_or_else(|_| "!".to_string());
                    let guild_id = msg.guild(&ctx.cache).await.unwrap().id;

                    if let Some(serde_json::Value::String(guild_prefix)) =
                        prefixes.get(guild_id.0.to_string())
                    {
                        Some(guild_prefix.clone())
                    } else {
                        Some(default_prefix)
                    }
                })
            })
            .prefix("")
        })
        .group(&COMMANDS_GROUP);

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let mut client = Client::builder(token)
        .event_handler(SerenityHandler)
        .framework(framework)
        .register_songbird()
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}
