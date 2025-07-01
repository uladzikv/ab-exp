use abexp::config::Config;
use abexp::domain::experiment::service::Service;
use abexp::inbound::http::{HttpServer, HttpServerConfig};
use abexp::outbound::sqlite::Sqlite;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::from_env()?;

    tracing_subscriber::fmt::init();

    let sqlite = Sqlite::new(&config.database_url).await?;
    let experiment_service = Service::new(sqlite);

    let server_config = HttpServerConfig {
        port: &config.server_port,
        auth_token: &config.auth_token,
    };

    let http_server = HttpServer::new(experiment_service, server_config).await?;

    http_server.run().await
}
