use tracing::subscriber::set_global_default;
use tracing::Subscriber;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry, fmt::MakeWriter};
// Compose multiple layers into a `tracing`s subscriber.
//
// # Implementation Notes
// We are using `impl Suscriber` as return type to avoid having to
// spell out the actual type of the returned subscriber, which is
// indeed quite complex.
// We need to explicity call out that the returned subscriber is
// `Send` and `Sync` to make it possible to pass it to `ini_subscriber`
// later on.
pub fn get_subscriber<Sink>(name: String, env_filter: String, sink: Sink) -> impl Subscriber + Send + Sync 
    where
        Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));
    let formatting_layer = BunyanFormattingLayer::new(name, sink);
    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

// Register a subscriber as global default to process span data
// It should only be called once!
// `init` does call `set_logger`, so this is all we need to do.
// We are fallingback to printing all logs at info-level or above
// if the RUST_LOG environment variable has not been set
// env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    LogTracer::init().expect("Failed to set logger");
    set_global_default(subscriber).expect("Failed to set subscriber");
}
