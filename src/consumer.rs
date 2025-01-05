use futures_util::{task::LocalSpawnExt, StreamExt};
use lapin::{options::*, types::FieldTable, Channel, Connection, ConnectionProperties, Consumer};
use serde_json::Error;
use tracing::{debug, error, info};

use crate::{db::primary_op, user::GeneralParams};

/// Connect to RabbitMQ and return the connection.
pub async fn connect_to_rabbitmq(addr: &str) -> lapin::Result<Connection> {
    let conn = Connection::connect(addr, ConnectionProperties::default()).await?;
    info!("Connected to RabbitMQ");
    Ok(conn)
}

/// Declare a queue on the given channel.
pub async fn declare_queue(channel: &Channel, queue_name: &str) -> lapin::Result<()> {
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

fn process_message(_data: &[u8]) {
    let result: Result<GeneralParams, Error> = serde_json::from_slice(_data);
    let client = primary_op::create_connection();
    match (result, client) {
        (Ok(dat), Ok(mut client)) => {
            // info!("Inserting values into database. {:?}", &dat.to_string())
            info!("Inserting values into database.");
            match primary_op::insert_into_table(client, dat) {
                Ok(_) => {
                    info!("Values inserted successfully!")
                }
                Err(err) => {
                    error!("Insertion failed. {:?}", err);
                }
            }
        }
        (Err(err_parser), Err(err_connection)) => {
            error!("Parser and Connection to database failed.")
        }
        _ => debug!("Gotcha!"),
    }
}

pub async fn consume_messages(consumer: Consumer, channel: Channel) -> lapin::Result<()> {
    info!("Consumer started");

    // Keep listening for messages until the consumer is stopped or the connection is closed.
    consumer
        .for_each(|delivery| async {
            if let Ok(delivery) = delivery {
                info!(
                    "Received message: {:?}",
                    std::str::from_utf8(&delivery.data)
                );

                process_message(&delivery.data);

                // Acknowledge the message after processing it
                if let Err(e) = channel
                    .basic_ack(delivery.delivery_tag, BasicAckOptions::default())
                    .await
                {
                    error!("Failed to ack message: {:?}", e);
                }
            } else {
                error!("Error receiving message.");
            }
        })
        .await;

    Ok(())
}

pub async fn start_consumer(channel: Channel, queue_name: &str) -> lapin::Result<()> {
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

// /// Spawn a consumer for the given queue.
// pub async fn spawn_consumer(
//     channel: Channel,
//     queue_name: &str,
//     consumer_tag: &str,
//     spawner: futures_executor::LocalSpawner,
// ) -> lapin::Result<()> {
//     let consumer = channel
//         .basic_consume(
//             queue_name,
//             consumer_tag,
//             BasicConsumeOptions::default(),
//             FieldTable::default(),
//         )
//         .await?;

//     spawner
//         .spawn_local({
//             let queue_name = queue_name.to_string(); // Clone `queue_name` into a `String` with `'static` lifetime
//             async move {
//                 info!("Consumer started for queue: {}", queue_name);
//                 consumer
//                     .for_each(|delivery| async {
//                         if let Ok(delivery) = delivery {
//                             info!(
//                                 "Received message: {:?}",
//                                 std::str::from_utf8(&delivery.data)
//                             );
//                             channel
//                                 .basic_ack(delivery.delivery_tag, BasicAckOptions::default())
//                                 .await
//                                 .unwrap();
//                         }
//                     })
//                     .await;
//             }
//         })
//         .expect("Failed to spawn consumer task");

//     Ok(())
// }
