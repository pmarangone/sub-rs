use futures_executor::LocalPool;

mod consumer;
mod custom_tracing;

use consumer::*;

fn main() -> lapin::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let addr = std::env::var("AMQP_ADDR").unwrap_or_else(|_| "amqp://127.0.0.1:5672/%2f".into());
    let mut executor = LocalPool::new();

    executor.run_until(async {
        let conn = connect_to_rabbitmq(&addr).await?;

        let publisher_channel = conn.create_channel().await?;
        let consumer_channel = conn.create_channel().await?;

        declare_queue(&publisher_channel, "hello").await?;

        // spawn_consumer(consumer_channel, "hello", "my_consumer", spawner).await
        start_consumer(consumer_channel, "hello").await
    })
}
