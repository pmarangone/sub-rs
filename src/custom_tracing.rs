use tracing::Level;
use tracing_appender::rolling;
use tracing_subscriber::{fmt, EnvFilter, Registry};

use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;

pub fn write_tracing_to_file() {
    let file_appender = rolling::hourly("./logs/", "example.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // Create a console layer with ANSI escape sequences (colors) enabled
    let console_layer = fmt::Layer::new()
        .with_writer(std::io::stdout) // Write to console
        .with_ansi(true); // Enable ANSI formatting for colors

    // Create a file layer without ANSI escape sequences (plain text)
    let file_layer = fmt::Layer::new()
        .with_writer(non_blocking) // Write to the file
        .with_ansi(false); // Disable ANSI formatting for plain text logs

    // Build the subscriber with both layers
    let subscriber = Registry::default()
        .with(EnvFilter::from_default_env().add_directive(Level::TRACE.into()))
        .with(console_layer)
        .with(file_layer);

    // Set the global default subscriber
    tracing::subscriber::set_global_default(subscriber).expect("Unable to set a global subscriber");
}
