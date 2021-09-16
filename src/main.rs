use serenity::{
    async_trait,
    client::{Context, EventHandler},
    framework::{standard::macros::group, StandardFramework},
    model::gateway::Ready,
    Client,
};
use songbird::SerenityInit;
use std::env;

use parrot::commands::{
    now_playing::*, pause::*, play::*, queue::*, repeat::*, resume::*, seek::*, skip::*, summon::*,
};

#[group]
#[commands(summon, play, pause, resume, seek, skip, now_playing, queue, repeat)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Failed to load .env file");

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("."))
        .group(&GENERAL_GROUP);

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let mut client = Client::builder(token)
        .event_handler(Handler)
        .framework(framework)
        .register_songbird()
        .await
        .expect("Error creating client");

    // Start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}
