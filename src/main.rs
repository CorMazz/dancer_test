mod config;
mod router;
mod auth;
mod views;
mod filters;
mod exam;

use config::{GoogleOAuthConfig, SecretsConfig};
use exam::{handlers::parse_test_definition_from_str, models::{SMTPConfig, TestDefinitionYaml}};
use lettre::{transport::smtp::authentication::Credentials, AsyncSmtpTransport, Tokio1Executor};
use lettre::transport::smtp::PoolConfig;
use oauth2::reqwest;
use std::{fs::File, io::Read, sync::Arc, time::Duration};

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
    smtp_config: Option<SMTPConfig>,
    smtp_mailer: Option<AsyncSmtpTransport<Tokio1Executor>>,
    google_oauth_config: Option<GoogleOAuthConfig>,
    http_client: reqwest::Client,
    test_configurations: TestDefinitionYaml,
}


#[tokio::main]
async fn main() {

    dotenv().ok();

    let config = SecretsConfig::init();

    let smtp_config = SMTPConfig::init();
    let smtp_mailer: Option<AsyncSmtpTransport<Tokio1Executor>> = smtp_config.as_ref().and_then(|config| {
        let creds = Credentials::new(
            config.user_login.clone(),
            config.user_password.clone(),
        );
    
        match AsyncSmtpTransport::<Tokio1Executor>::relay(&config.server_host) {
            Ok(transport) => Some(
                transport
                    .credentials(creds)
                    .pool_config(
                        PoolConfig::new()
                            .max_size(10)
                            .idle_timeout(Duration::from_secs(60))
                    )
                    .build()
            ),
            Err(e) => {
                eprintln!("Error: Unable to connect to email server: {}", e);
                None
            }
        }
    });

    let google_oauth_config = GoogleOAuthConfig::init();

    let file_path = "test_definitions.yaml";
    let mut file = File::open(file_path).expect(&format!("couldn't open file: {}", file_path));
    let mut yaml_string = String::new();
    file.read_to_string(&mut yaml_string).expect(&format!("Couldn't read file '{}' to string. This should work...", file_path));
    let mut tests = parse_test_definition_from_str(&yaml_string).expect("Error parsing test_definition.yaml");

    for test in &mut tests.tests {
        test.validate().expect("Invalid test definition");
    }

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

    // For Google OAuth flow
    let http_client = reqwest::ClientBuilder::new()
        // Following redirects opens the client up to SSRF vulnerabilities.
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("Client should build");

    let cors = CorsLayer::new()
        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    let app = create_router(Arc::new(AppState {
        db: pool.clone(),
        env: config.clone(),
        smtp_config,
        smtp_mailer,
        google_oauth_config,
        http_client,
        redis_client: redis_client.clone(),
        test_configurations: tests,
    }))
    .layer(cors);

    println!("🚀 Server started successfully on port {}", config.server_port);
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", config.server_port)).await.unwrap();
    axum::serve(listener, app).await.unwrap()
}