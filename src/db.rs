use std::env;

use dotenv::dotenv;
use mongodb::{options::ClientOptions, Client};
use tracing::info;

pub async fn connect_to_db() -> Client
{
    dotenv().ok();

    let mongo_uri = env::var("MONGO_URI").expect("MONGO_URI no está configurado");

    let client_options = ClientOptions::parse(&mongo_uri).await
                                                         .expect("Failed to parse options");
    let mongo_client = Client::with_options(client_options).expect("Failed to initialize MongoDB client");

    info!("Connected to MongoDB");

    mongo_client
}
