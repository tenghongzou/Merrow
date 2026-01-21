#[tokio::main]
async fn main() {
    dotenvy::from_filename(".env.local").ok();
    dotenvy::dotenv().ok();
    merrow::app::logging::init();

    let args: Vec<String> = std::env::args().collect();
    let (config_path, addr, pg_enabled) = parse_args(&args).unwrap_or_else(|err| {
        eprintln!("error: {err}");
        std::process::exit(1);
    });

    if let Err(err) = merrow::app::ui_server::run(&addr, &config_path, pg_enabled).await {
        eprintln!("error: {}", err.message);
        std::process::exit(1);
    }
}

fn parse_args(args: &[String]) -> Result<(String, String, Option<bool>), String> {
    let mut config_path = "config.toml".to_string();
    let mut addr = std::env::var("MERROW_UI_ADDR").unwrap_or_else(|_| "127.0.0.1:8088".to_string());
    let mut pg_enabled = None;

    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--config" | "-c" => {
                let value = args.get(index + 1).ok_or("missing value for --config")?;
                config_path = value.to_string();
                index += 2;
            }
            "--addr" => {
                let value = args.get(index + 1).ok_or("missing value for --addr")?;
                addr = value.to_string();
                index += 2;
            }
            "--pg-enabled" => {
                let value = args.get(index + 1).ok_or("missing value for --pg-enabled")?;
                pg_enabled = Some(parse_bool(value)?);
                index += 2;
            }
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
            }
            unknown => {
                return Err(format!("unknown argument: {unknown}"));
            }
        }
    }

    Ok((config_path, addr, pg_enabled))
}

fn parse_bool(value: &str) -> Result<bool, String> {
    match value.trim().to_lowercase().as_str() {
        "1" | "true" | "yes" => Ok(true),
        "0" | "false" | "no" => Ok(false),
        _ => Err("invalid bool value".to_string()),
    }
}

fn print_usage() {
    println!("usage: merrow_ui [--config <path>] [--addr <host:port>] [--pg-enabled <bool>]");
    println!("  -c, --config   Path to config.toml (default: config.toml)");
    println!("      --addr     Bind address (default: 127.0.0.1:8088 or MERROW_UI_ADDR)");
    println!("      --pg-enabled  Enable PGSQL access (true/false)");
}
