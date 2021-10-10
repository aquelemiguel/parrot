use serenity::{Client, async_trait, client::{Context, EventHandler}, framework::{standard::macros::group, StandardFramework}, model::{gateway::Ready, prelude::Activity}};
use songbird::SerenityInit;
use std::env;

use parrot::commands::{
    clear::*, leave::*, now_playing::*, pause::*, play::*, queue::*, repeat::*, resume::*, seek::*,
    shuffle::*, skip::*, stop::*, summon::*, remove::*,
};

#[group]
#[commands(
    clear,
    leave,
    now_playing,
    pause,
    play,
    queue,
    repeat,
    resume,
    seek,
    shuffle,
    skip,
    stop,
    summon,
    remove
)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
        ctx.set_activity(Activity::listening("!play")).await;
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Failed to load .env file");

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("!"))
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
