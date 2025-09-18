use std::fs::File;
use tracing_loki::url::Url;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};

pub fn setup_logging(module_name: &str) {
    let others_filter = format!("{module_name}=debug");

    let stdout_log = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stdout)
        .with_target(true)
        .with_level(true)
        .with_filter(EnvFilter::new(&others_filter));

    let file = File::create(format!("{module_name}/log.txt")).expect("failed to create log file");
    let file_log = tracing_subscriber::fmt::layer()
        .with_writer(file)
        .with_ansi(false)
        .with_target(true)
        .with_level(true)
        .with_filter(EnvFilter::new(&others_filter));

    let (loki, loki_task) = tracing_loki::builder()
        .label("application", module_name)
        .expect("invalid loki label")
        .extra_field("pid", std::process::id().to_string())
        .expect("invalid loki field")
        .build_url(Url::parse("http://localhost:3100").expect("invalid loki url"))
        .expect("could not build loki");

    tracing_subscriber::registry()
        .with(stdout_log)
        .with(file_log)
        .with(loki.with_filter(EnvFilter::new(&others_filter)))
        .init();

    tokio::spawn(loki_task);
}
