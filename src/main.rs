mod config;
mod router;
mod auth;
mod views;
mod filters;
mod exam;

use config::SecretsConfig;
use exam::{handlers::parse_test_definition, models::TestDefinition};
use std::{fs::File, io::Read, sync::Arc};

use axum::http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    HeaderValue, Method,
};
use dotenv::dotenv;
use redis::Client;
use router::create_router;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use tower_http::cors::CorsLayer;

pub struct AppState {
    db: Pool<Postgres>,
    env: SecretsConfig,
    redis_client: Client,
    leader_test: TestDefinition,
    follower_test: TestDefinition,
}


#[tokio::main]
async fn main() {

    dotenv().ok();

    let config = SecretsConfig::init();

    let leader_test = parse_test_definition("leader_test.yaml").expect("Error parsing leader_test.yaml");
    println!("{:#?}", leader_test.clone());
    let follower_test = parse_test_definition("follower_test.yaml").expect("Error parsing follower_test.yaml");

    let pool = match PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await
    {
        Ok(pool) => {
            println!("✅ Connection to the database is successful!");
            pool
        }
        Err(err) => {
            println!("🔥 Failed to connect to the database: {:?}", err);
            std::process::exit(1);
        }
    };

    // Determine if a new test was loaded

    let redis_client = match Client::open(config.redis_url.to_owned()) {
        Ok(client) => {
            println!("✅ Connection to the redis server is successful!");
            client
        }
        Err(e) => {
            println!("🔥 Error connecting to Redis: {}", e);
            std::process::exit(1);
        }
    };

    let cors = CorsLayer::new()
        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    let app = create_router(Arc::new(AppState {
        db: pool.clone(),
        env: config.clone(),
        redis_client: redis_client.clone(),
        leader_test: leader_test,
        follower_test: follower_test,
    }))
    .layer(cors);

    println!("🚀 Server started successfully on port {}", config.server_port);
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", config.server_port)).await.unwrap();
    axum::serve(listener, app).await.unwrap()
}