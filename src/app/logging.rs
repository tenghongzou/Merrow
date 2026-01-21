use tracing_subscriber::EnvFilter;

pub fn init() {
    let filter = env_filter();
    let format = log_format();
    let builder = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false);

    let result = match format.as_str() {
        "json" => builder.json().try_init(),
        _ => builder.try_init(),
    };

    let _ = result;
}

fn env_filter() -> EnvFilter {
    let override_level = std::env::var("MERROW_LOG")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| std::env::var("RUST_LOG").ok());

    match override_level {
        Some(value) => EnvFilter::new(value),
        None => EnvFilter::new("info"),
    }
}

fn log_format() -> String {
    std::env::var("MERROW_LOG_FORMAT")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "plain".to_string())
}
