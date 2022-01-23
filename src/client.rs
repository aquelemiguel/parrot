use songbird::SerenityInit;
use std::{collections::HashMap, env, error::Error};

use crate::{
    guild::{GuildCacheMap, GuildSettingsMap},
    handlers::SerenityHandler,
};

pub struct Client {
    client: serenity::Client,
}

impl Client {
    pub async fn default() -> Result<Client, Box<dyn Error>> {
        let token = env::var("DISCORD_TOKEN")?;
        Client::new(token).await
    }

    pub async fn new(token: String) -> Result<Client, Box<dyn Error>> {
        let application_id = env::var("DISCORD_APPID")?.parse()?;

        let client = serenity::Client::builder(token)
            .event_handler(SerenityHandler)
            .application_id(application_id)
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
