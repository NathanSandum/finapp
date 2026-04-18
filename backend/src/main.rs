use diesel::r2d2::{self, ConnectionManager};
use diesel::SqliteConnection;

#[tokio::main]
async fn main() {
    let manager = ConnectionManager::<SqliteConnection>::new("finapp.db");
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("failed to create db pool");

    let app = backend::build_router(pool);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("Backend running on http://127.0.0.1:3000");
    axum::serve(listener, app).await.unwrap();
}
