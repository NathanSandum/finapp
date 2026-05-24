use backend::config::{Config, Mode};
use backend::{database, seed};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,tower_http=debug")),
        )
        .init();

    let config = Config::load().expect("config load failed");

    let db = config.database.connect();
    if config.mode == Mode::Dev {
        database::run_migrations(&db);
        seed::populate(&db, &config).expect("dev seed failed");
    }

    let app = backend::build_router(db, config.mode);

    let bind = format!("127.0.0.1:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&bind).await.unwrap();

    tracing::info!(
        bind = %bind,
        mode = ?config.mode,
        database = %config.database,
        "backend listening"
    );

    if config.mode == Mode::Dev {
        tracing::info!(
            email = seed::DEV_USER_EMAIL,
            password = seed::DEV_USER_PASSWORD,
            workspace = %seed::DEV_WORKSPACE_ID,
            api_base = %format!("http://{}", bind),
            "dev credentials"
        );
    }

    axum::serve(listener, app).await.unwrap();
}
