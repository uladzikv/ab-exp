use std::sync::Arc;

use anyhow::Context;
use axum::Router;
use axum::routing::{get, patch, post};
use tokio::net;

use crate::domain::experiment::ports::ExperimentService;
use crate::inbound::http::handlers::{
    create_experiment::create_experiment, get_experiments::get_experiments,
    get_statistics::get_statistics, patch_experiment::patch_experiment,
};

mod handlers;
mod responses;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpServerConfig<'a> {
    pub port: &'a str,
    pub auth_token: &'a str,
}

#[derive(Debug, Clone)]
struct AppState<ES: ExperimentService> {
    experiment_service: Arc<ES>,
    auth_token: String,
}

pub struct HttpServer {
    router: axum::Router,
    listener: net::TcpListener,
}

impl HttpServer {
    pub async fn new(
        experiment_service: impl ExperimentService,
        config: HttpServerConfig<'_>,
    ) -> anyhow::Result<Self> {
        let trace_layer = tower_http::trace::TraceLayer::new_for_http().make_span_with(
            |request: &axum::extract::Request<_>| {
                let uri = request.uri().to_string();
                tracing::info_span!("http_request", method = ?request.method(), uri)
            },
        );

        let state = AppState {
            experiment_service: Arc::new(experiment_service),
            auth_token: config.auth_token.to_string(),
        };

        let router = axum::Router::new()
            .nest("/api", api_routes())
            .layer(trace_layer)
            .with_state(state);

        let listener = net::TcpListener::bind(format!("0.0.0.0:{}", config.port))
            .await
            .with_context(|| format!("failed to listen on {}", config.port))?;

        Ok(Self { router, listener })
    }

    pub async fn run(self) -> anyhow::Result<()> {
        tracing::debug!("listening on {}", self.listener.local_addr().unwrap());
        axum::serve(self.listener, self.router)
            .await
            .context("received error from running server")?;

        Ok(())
    }
}

fn api_routes<ES: ExperimentService>() -> Router<AppState<ES>> {
    Router::new()
        .route("/experiments", get(get_experiments))
        .route("/experiments", post(create_experiment))
        .route("/experiments/{id}", patch(patch_experiment))
        .route("/statistics", get(get_statistics))
}
