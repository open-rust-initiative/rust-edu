use tokio::{net::TcpListener, signal};
use tracing_subscriber::{layer::SubscriberExt, fmt, util::SubscriberInitExt};
use waterdb::{server, config::Config};

#[tokio::main]
pub async fn main() -> waterdb::Result<()> {
    tracing_subscriber::registry().with(fmt::layer()).init();

    let args = clap::command!()
    .arg(
        clap::Arg::new("config")
            .short('c')
            .long("config")
            .help("Configuration file path")
            .default_value("config/waterdb.yaml"),
    )
    .get_matches();
    let cfg = Config::new(args.get_one::<String>("config").unwrap().as_ref())?;
    let data_path = std::path::Path::new(&cfg.data_dir);
    let default_ip = cfg.default_ip.as_str();
    let default_port = cfg.default_port.as_str();

    let addr = format!("{}:{}", default_ip, default_port);
    
    // Bind a TCP listener
    let listener = TcpListener::bind(&addr).await?;

    let _ = server::run(listener, signal::ctrl_c(), data_path).await;

    Ok(())
}