fn main() {
    dotenvy::from_filename(".env.local").ok();
    dotenvy::dotenv().ok();
    merrow::app::logging::init();
    if let Err(err) = merrow::app::cli::run() {
        eprintln!("error: {}", err.message);
        std::process::exit(1);
    }
}
