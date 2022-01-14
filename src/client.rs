use serde_json::Value;
use serenity::{client::Context, model::channel::Message};
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
use crate::{events::serenity_handler::SerenityHandler, strings::DEFAULT_PREFIX};

pub struct Client {
    client: serenity::Client,
}

impl Client {
    pub async fn default() -> Result<Client, Box<dyn Error>> {
        let token = env::var("DISCORD_TOKEN")?;
        Client::new(token).await
    }

    pub async fn new(token: String) -> Result<Client, Box<dyn Error>> {
        let application_id: u64 = env::var("DISCORD_APPID").unwrap().parse().unwrap();

        let client = serenity::Client::builder(token)
            .event_handler(SerenityHandler)
            .application_id(application_id)
            .register_songbird()
            .await?;

        Ok(Client { client })
    }

    pub async fn start(&mut self) -> Result<(), serenity::Error> {
        self.client.start().await
    }
}
