use crate::config::CONFIG;
use std::fs::File;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Layer};
use url::Url;

pub fn setup_logging(module_name: &str) {
    let others_filter = format!("{module_name}=debug");

    let stdout_log = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stdout)
        .with_target(true)
        .with_level(true)
        .with_filter(EnvFilter::new(&others_filter));

    std::fs::create_dir_all(module_name).expect("failed to create log directory");
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
        .build_url(Url::parse(&CONFIG.loki.url).expect("invalid loki url"))
        .expect("could not build loki");

    tracing::subscriber::set_global_default(
        tracing_subscriber::registry()
            .with(stdout_log)
            .with(file_log)
            .with(loki.with_filter(EnvFilter::new(&others_filter))),
    )
    .expect("failed to set global default");

    tokio::spawn(loki_task);
}
