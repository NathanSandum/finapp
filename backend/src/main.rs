use backend::config::Config;
use backend::database;

#[tokio::main]
async fn main() {
    let config = Config::load().expect("config load failed");
    let pool = database::build(&config);

    let app = backend::build_router(pool);

    let bind = format!("127.0.0.1:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&bind)
        .await
        .unwrap();

    println!(
        "Backend running on http://{} (mode: {:?}, database: {})",
        bind, config.mode, config.database
    );
    axum::serve(listener, app).await.unwrap();
}
