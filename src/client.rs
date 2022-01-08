use serde_json::Value;
use serenity::{
    client::Context,
    framework::{standard::macros::group, StandardFramework},
    model::channel::Message,
};
use songbird::SerenityInit;
use std::{env, error::Error};

use crate::commands::{
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
};
use crate::{
    events::serenity_handler::SerenityHandler, strings::DEFAULT_PREFIX, utils::get_prefixes,
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

pub struct Client {
    client: serenity::Client,
}

impl Client {
    pub async fn new(token: Option<String>) -> Result<Client, Box<dyn Error>> {
        let token = token.unwrap_or(env::var("DISCORD_TOKEN")?);

        let framework = StandardFramework::new()
            .configure(|c| {
                c.dynamic_prefix(|ctx, msg| Box::pin(Client::get_prefix(ctx, msg)))
                    .prefix("")
            })
            .group(&COMMANDS_GROUP);

        let client = serenity::Client::builder(token)
            .event_handler(SerenityHandler)
            .framework(framework)
            .register_songbird()
            .await?;

        Ok(Client { client })
    }

    pub async fn start(&mut self) -> Result<(), serenity::Error> {
        self.client.start().await
    }

    async fn get_prefix(ctx: &Context, msg: &Message) -> Option<String> {
        let prefixes = get_prefixes();
        let default_prefix = env::var("PREFIX").unwrap_or_else(|_| DEFAULT_PREFIX.to_string());
        let guild_id = msg.guild(&ctx.cache).await?.id;
        let guild_prefix = prefixes.get(guild_id.0.to_string());

        if let Some(Value::String(guild_prefix)) = guild_prefix {
            Some(guild_prefix.clone())
        } else {
            Some(default_prefix)
        }
    }
}
