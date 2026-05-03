use backend::config::{Config, Mode};
use backend::{database, seed};

#[tokio::main]
async fn main() {
    let config = Config::load().expect("config load failed");

    let db = config.database.connect();
    if config.mode == Mode::Dev {
        database::run_migrations(&db);
        seed::populate(&db, &config).expect("dev seed failed");
    }

    let app = backend::build_router(db);

    let bind = format!("127.0.0.1:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&bind)
        .await
        .unwrap();

    println!(
        "Backend running on http://{} (mode: {:?}, database: {})",
        bind, config.mode, config.database
    );

    if config.mode == Mode::Dev {
        if let Some(token) = config.dev_token.as_deref() {
            println!();
            println!("── Dev credentials ──────────────────────────────");
            println!("  VITE_DEV_TOKEN     = {token}");
            println!("  VITE_DEV_WORKSPACE = {}", seed::DEV_WORKSPACE_ID);
            println!("  VITE_API_BASE      = http://{}", bind);
            println!("─────────────────────────────────────────────────");
        }
    }

    axum::serve(listener, app).await.unwrap();
}
