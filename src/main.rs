use futures_executor::LocalPool;
use futures_util::{future::FutureExt, stream::StreamExt, task::LocalSpawnExt};
use lapin::{
    options::*, publisher_confirm::Confirmation, types::FieldTable, BasicProperties, Channel,
    Connection, ConnectionProperties, Consumer, Result,
};

use log::info;

fn main() -> Result<()> {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    let addr = std::env::var("AMQP_ADDR").unwrap_or_else(|_| "amqp://127.0.0.1:5672/%2f".into());
    let mut executor = LocalPool::new();
    let spawner = executor.spawner();

    executor.run_until(async {
        let conn = connect_to_rabbitmq(&addr).await?;

        let publisher_channel = conn.create_channel().await?;
        let consumer_channel = conn.create_channel().await?;

        declare_queue(&publisher_channel, "hello").await?;

        // spawn_consumer(consumer_channel, "hello", "my_consumer", spawner).await
        start_consumer(consumer_channel, "hello").await

        // publish_messages(publisher_channel, "hello").await
        // Ok(1)
    })
}

/// Connect to RabbitMQ and return the connection.
async fn connect_to_rabbitmq(addr: &str) -> Result<Connection> {
    let conn = Connection::connect(addr, ConnectionProperties::default()).await?;
    info!("Connected to RabbitMQ");
    Ok(conn)
}

/// Declare a queue on the given channel.
async fn declare_queue(channel: &Channel, queue_name: &str) -> Result<()> {
    channel
        .queue_declare(
            queue_name,
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;
    info!("Declared queue: {}", queue_name);
    Ok(())
}

async fn consume_messages(consumer: Consumer, channel: Channel) -> Result<()> {
    info!("Consumer started");

    // Keep listening for messages until the consumer is stopped or the connection is closed.
    consumer
        .for_each(|delivery| async {
            if let Ok(delivery) = delivery {
                info!(
                    "Received message: {:?}",
                    std::str::from_utf8(&delivery.data)
                );

                // Acknowledge the message after processing it
                if let Err(e) = channel
                    .basic_ack(delivery.delivery_tag, BasicAckOptions::default())
                    .await
                {
                    eprintln!("Failed to ack message: {:?}", e);
                }
            } else {
                eprintln!("Error receiving message.");
            }
        })
        .await;

    Ok(())
}

async fn start_consumer(channel: Channel, queue_name: &str) -> Result<()> {
    let consumer = channel
        .basic_consume(
            queue_name,
            "consumer_tag",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    consume_messages(consumer, channel).await
}

/// Spawn a consumer for the given queue.
async fn spawn_consumer(
    channel: Channel,
    queue_name: &str,
    consumer_tag: &str,
    spawner: futures_executor::LocalSpawner,
) -> Result<()> {
    let consumer = channel
        .basic_consume(
            queue_name,
            consumer_tag,
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    spawner
        .spawn_local({
            let queue_name = queue_name.to_string(); // Clone `queue_name` into a `String` with `'static` lifetime
            async move {
                info!("Consumer started for queue: {}", queue_name);
                consumer
                    .for_each(|delivery| async {
                        if let Ok(delivery) = delivery {
                            info!(
                                "Received message: {:?}",
                                std::str::from_utf8(&delivery.data)
                            );
                            channel
                                .basic_ack(delivery.delivery_tag, BasicAckOptions::default())
                                .await
                                .unwrap();
                        }
                    })
                    .await;
            }
        })
        .expect("Failed to spawn consumer task");

    Ok(())
}

/// Publish messages to the given queue in a loop.
async fn publish_messages(channel: Channel, queue_name: &str) -> Result<()> {
    let payload = b"Hello world!";
    loop {
        let confirm = channel
            .basic_publish(
                "",
                queue_name,
                BasicPublishOptions::default(),
                payload, // Pass as a reference
                BasicProperties::default(),
            )
            .await?
            .await?;

        assert_eq!(confirm, Confirmation::NotRequested);
        info!(
            "Message published: {}",
            std::str::from_utf8(payload).unwrap()
        );
    }
}
