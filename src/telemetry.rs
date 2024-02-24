use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::{
    layer::SubscriberExt, util::SubscriberInitExt, EnvFilter,
};
pub fn setup_tracing<Sink>(level: &str, sink: Sink)
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new(format!(
            "axum_newsletter={},axum::rejection=trace,tower-http={}",
            level, level
        ))
    });
    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer().with_writer(sink).json())
        .try_init()
        .expect("Failed to set subscriber.");
}
