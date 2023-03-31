use std::env;
use std::sync::Arc;

use mongodb::options::ClientOptions;
use mongodb::bson::{doc, Document};

use serenity::model::event::ResumedEvent;
use tracing::{error, info};

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::{GatewayIntents, Ready};
use serenity::prelude::*;

struct Database;

impl TypeMapKey for Database {
    type Value = Arc<mongodb::Database>;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        //maki: 235657818194051072
        if msg.author.id == 235657818194051072 {
            let data = ctx.data.read().await;
            if let Some(db) = data.get::<Database>() {
                let collection = db.collection::<Document>("neurons");
                match collection.insert_one(doc!{ "message": msg.content}, None).await {
                    Ok(_) => (),
                    Err(e) => error!("{:?}", e),
                };

            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().expect("Failed to load .env file");
    tracing_subscriber::fmt::init();

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let connection_string = env::var("DB_CONNECTION_STRING").expect("Expected a mongodb connection uri");

    let client_options = ClientOptions::parse(connection_string).await?;
    let dbclient = mongodb::Client::with_options(client_options)?;

    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<Database>(Arc::new(dbclient.database("makibrain")));
    }
    let shard_manager = client.shard_manager.clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Could not register ctrl+c handler");
        shard_manager.lock().await.shutdown_all().await;
    });

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }
    Ok(())
}
