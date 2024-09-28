mod config;
mod router;
mod auth;
mod views;
mod filters;
mod exam;

use config::SecretsConfig;
use exam::{handlers::parse_test_definition, models::{SMTPConfig, TestDefinitionYaml}};
use lettre::{transport::smtp::authentication::Credentials, AsyncSmtpTransport, Tokio1Executor};
use uuid::Uuid;
use std::sync::Arc;

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
    test_configurations: TestDefinitionYaml,
}


#[tokio::main]
async fn main() {

    dotenv().ok();

    let config = SecretsConfig::init();

    let smtp_config = match (
        &config.smtp_server_host,
        &config.smtp_user_login,
        &config.smtp_user_password,
        &config.smtp_user_email,
    ) {
        (host, user, password, email) if host.is_empty() && user.is_empty() && password.is_empty() && email.is_empty() => {
            println!("\nEmail functionality disabled since all SMTP environment variables were left blank.");
            None
        },
        (host, _, _, _) if host.is_empty() => {
            println!("\nEmail functionality disabled since the SMTP_SERVER_HOST environment variable was left blank.");
            None
        },
        (_, user, _, _) if user.is_empty() => {
            println!("\nEmail functionality disabled since the SMTP_USER_LOGIN environment variable was left blank.");
            None
        },
        (_, _, password, _) if password.is_empty() => {
            println!("\nEmail functionality disabled since the SMTP_USER_PASSWORD environment variable was left blank.");
            None
        },
        (_, _, _, email) if email.is_empty() => {
            println!("\nEmail functionality disabled since the SMTP_USER_EMAIL environment variable was left blank.");
            None
        },
        (host, user, password, email) if !host.is_empty() && !user.is_empty() && !password.is_empty() && !email.is_empty() => {
            
            println!("\nEmail functionality is enabled with the following settings:\n\tServer: {}\n\tUsername: {}\n\tEmail: {}\n", host, user, email);
            
            Some(SMTPConfig {
                server_host: host.to_string(),
                user_login: user.to_string(),
                user_password: password.to_string(),
                user_email: email.to_string(),
            })
        },
        _ => {
            panic!("Something odd is happening in the SMTP settings creation. Ensure you're feeding strings in as the environment variables.");
        },
    };

    // I can't figure out how to make this more idiomatic. 
    let smtp_mailer: Option<AsyncSmtpTransport<Tokio1Executor>> = match &smtp_config {
        Some(config) => {
            let creds = Credentials::new(
                config.user_login.clone(),
                config.user_password.clone(),
            );

            match AsyncSmtpTransport::<Tokio1Executor>::relay(&config.server_host) {
                Ok(transport) => Some(transport.credentials(creds).build()),
                Err(e) => {
                    eprintln!("Error: Unable to connect to email server: {}", e);
                    None
                }
            }

        },
        None => None, // This feels wrong and like I could make it less verbose.
    };


    let tests = parse_test_definition("test_definitions.yaml").expect("Error parsing test_definition.yaml");

    for test in &tests.tests {
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
        smtp_config,
        smtp_mailer,
        redis_client: redis_client.clone(),
        test_configurations: tests,
    }))
    .layer(cors);

    println!("🚀 Server started successfully on port {}", config.server_port);
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", config.server_port)).await.unwrap();
    axum::serve(listener, app).await.unwrap()
}