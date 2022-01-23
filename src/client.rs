use serenity::{
    model::{channel::Message, id::GuildId},
    prelude::{RwLock, TypeMapKey},
};
use songbird::SerenityInit;
use std::{collections::HashMap, env, error::Error, sync::Arc};

use crate::{handlers::SerenityHandler, settings::GuildSettingsMap};

pub struct Client {
    client: serenity::Client,
}

pub struct GuildQueueInteractions;
type QueueMessage = (Message, Arc<RwLock<usize>>);

impl TypeMapKey for GuildQueueInteractions {
    type Value = HashMap<GuildId, Vec<QueueMessage>>;
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
        data.insert::<GuildQueueInteractions>(HashMap::default());
        data.insert::<GuildSettingsMap>(HashMap::default());
        drop(data);

        Ok(Client { client })
    }

    pub async fn start(&mut self) -> Result<(), serenity::Error> {
        self.client.start().await
    }
}
