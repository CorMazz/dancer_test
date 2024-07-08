use axum::{routing::get, Router};

/// https://vitor.ws/blog/how_use_axum_askama_htmx

async fn root() -> &'static str {
    "Hello from Axum"
}

#[tokio::main]
async fn main() -> Result<(), ()> {
    tracing_subscriber::fmt().init();
    let port = "5000";

    tracing::info!("router initialized, now listening on port {}", port);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .unwrap();

    let app = Router::new().route("/", get(root));

    axum::serve(listener, app).await.unwrap();

    Ok(())
}