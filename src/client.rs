use serenity::model::gateway::GatewayIntents;
use songbird::SerenityInit;

use std::{collections::HashMap, env, error::Error};

use crate::{
    guild::{cache::GuildCacheMap, settings::GuildSettingsMap},
    handlers::SerenityHandler,
};

pub struct Client {
    client: serenity::Client,
}

impl Client {
    pub async fn default() -> Result<Client, Box<dyn Error>> {
        let token = env::var("DISCORD_TOKEN").expect("Fatality! DISCORD_TOKEN not set!");
        Client::new(token).await
    }

    pub async fn new(token: String) -> Result<Client, Box<dyn Error>> {
        let gateway_intents = GatewayIntents::non_privileged();

        let client = serenity::Client::builder(token, gateway_intents)
            .event_handler(SerenityHandler)
            .register_songbird()
            .await?;

        let mut data = client.data.write().await;
        data.insert::<GuildCacheMap>(HashMap::default());
        data.insert::<GuildSettingsMap>(HashMap::default());
        drop(data);

        Ok(Client { client })
    }

    pub async fn start(&mut self) -> Result<(), serenity::Error> {
        self.client.start().await
    }
}
