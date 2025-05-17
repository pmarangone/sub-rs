use anyhow::{Error, Result as ResultHow};
use futures_executor::LocalPool;

mod consumer;
mod custom_tracing;
mod db;
mod models;

use consumer::*;
use db::primary_op::{create_connection, create_table};
use lapin::protocol::queue;
use models::user;
use postgres::Client;
use tracing::error;

fn db_init() -> Result<Client, Error> {
    let client = create_connection();

    match client {
        Ok(mut c) => match create_table(&mut c) {
            Ok(_) => Ok(c),
            Err(err) => {
                error!("{}", err);
                ResultHow::Err(Error::msg("Failed to created table person."))
            }
        },
        Err(err) => {
            error!("{}", err);
            ResultHow::Err(Error::msg("Connection to the database failed."))
        }
    }
}

fn main() -> lapin::Result<()> {
    println!("Hello world!");
    tracing_subscriber::fmt::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // match db_init() {
    //     Ok(_client) => {
    tracing::info!("Database connected successfully.");

    let addr = std::env::var("RABBITMQ_URI").unwrap_or_else(|_| "amqp://127.0.0.1:5672/%2f".into());
    let queue_name = std::env::var("QUEUE_NAME").unwrap_or_else(|_| "notebooklm".into());
    let mut executor = LocalPool::new();

    executor.run_until(async {
        let conn = connect_to_rabbitmq(&addr).await?;

        // let publisher_channel = conn.create_channel().await?;
        let consumer_channel = conn.create_channel().await?;

        // declare_queue(&publisher_channel, &queue_name).await?;

        // spawn_consumer(consumer_channel, queue_name, "my_consumer", spawner).await
        start_consumer(consumer_channel, &queue_name).await
    })
    // }
    // Err(err) => {
    //     error!("Application initialization failed: {}", err);
    //     std::process::exit(1);
    // }
    // }
}
