use serenity::{
    async_trait,
    client::{Context, EventHandler},
    framework::{standard::macros::group, StandardFramework},
    model::{
        gateway::Ready,
        id::GuildId,
        prelude::{Activity, VoiceState},
    },
    Client,
};
use songbird::SerenityInit;
use std::env;

use parrot::{
    commands::{
        clear::*, leave::*, now_playing::*, pause::*, play::*, playtop::*, prefix::*, queue::*,
        remove::*, repeat::*, resume::*, seek::*, shuffle::*, skip::*, stop::*, summon::*,
    },
    utils::get_prefixes,
};

use parrot::commands::genius::{explain::*, lyrics::*};

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
    summon
)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("🦜 {} is connected!", ready.user.name);
        ctx.set_activity(Activity::listening("!play")).await;
    }

    async fn voice_state_update(
        &self,
        ctx: Context,
        guild: Option<GuildId>,
        _old: Option<VoiceState>,
        new: VoiceState,
    ) {
        if new.user_id == ctx.http.get_current_user().await.unwrap().id && !new.deaf {
            guild
                .unwrap()
                .edit_member(&ctx.http, new.user_id, |n| n.deafen(true))
                .await
                .unwrap();
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let framework = StandardFramework::new()
        .configure(|c| {
            c.dynamic_prefix(|ctx, msg| {
                Box::pin(async move {
                    let prefixes = get_prefixes();

                    let guild_id = msg.guild(&ctx.cache).await.unwrap().id;

                    if let Some(serde_json::Value::String(prefix)) =
                        prefixes.get(guild_id.0.to_string())
                    {
                        // Command from guild_id.0 with prefix
                        Some(prefix.clone())
                    } else {
                        // Command from guild_id.0 with default prefix
                        Some("!".to_string())
                    }
                })
            })
            .prefix("")
        })
        .group(&GENERAL_GROUP);

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let mut client = Client::builder(token)
        .event_handler(Handler)
        .framework(framework)
        .register_songbird()
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}
