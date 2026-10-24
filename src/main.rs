use std::net::SocketAddr;

use axum::http::{header, HeaderName, HeaderValue, Method};
use tower_http::{
    cors::CorsLayer,
    limit::RequestBodyLimitLayer,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use secure_bank::{
    api::create_router,
    config::Config,
    infrastructure::{create_pool, run_migrations},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Logging
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            "secure_bank=debug,tower_http=info,sqlx=warn".into()
        }))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env()?;
    tracing::info!(host = %config.host, port = config.port, "starting secure-bank");

    // Database
    let pool = create_pool(&config).await?;
    run_migrations(&pool).await?;
    tracing::info!("database migrations applied");

    // CORS – tight for production; permissive for local skeleton
    let cors = CorsLayer::new()
        .allow_origin(HeaderValue::from_static("http://localhost:3000"))
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION, HeaderName::from_static("x-user-id")]);

    let app = create_router(pool)
        .layer(RequestBodyLimitLayer::new(64 * 1024)) // 64 KiB
        .layer(TraceLayer::new_for_http())
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(cors);

    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    tracing::info!(%addr, "listening");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
