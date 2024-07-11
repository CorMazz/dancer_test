use askama_axum::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get, 
    Router,
    extract::Form,
};
use serde::Deserialize;

/// https://vitor.ws/blog/how_use_axum_askama_htmx

#[derive(Template)]
#[template(path = "root.html")]
struct RootTemplate {}

async fn root() -> impl IntoResponse {
    let root: RootTemplate = RootTemplate { };
    (StatusCode::OK, Html(root.render().unwrap()))
}


// #########################################################################################################################################
// Leader Test Template
// #########################################################################################################################################

#[derive(Template)]
#[template(path = "leader_test.html")]
struct LeaderTestTemplate {
    points: Vec<usize>,
    dance_moves: Vec<&'static str>,
}

async fn leader_test_get() -> impl IntoResponse {
    let points: Vec<usize> = vec![3, 2, 1];

    let leader_test: LeaderTestTemplate = LeaderTestTemplate {
        points: points,
        dance_moves: vec!["Left Side Pass", "Right Side Pass", "Whip", "Reverse Whip"]
    };

    (StatusCode::OK, Html(leader_test.render().unwrap()))
}

// Later this can be refactored with a procedural macro to accept a global dance moves list that automagically
// generates this struct, thus reducing repetition of the dance move points list
#[derive(Debug, Deserialize)]
struct DanceMovePoints {
    left_side_pass: usize,
    right_side_pass: usize,
    whip: usize,
    reverse_whip: usize,
}

async fn leader_test_post(Form(data): Form<DanceMovePoints>) {
    println!("{:?}", data);
}


// #########################################################################################################################################
// Main
// #########################################################################################################################################


#[tokio::main]
async fn main() -> Result<(), ()> {
    tracing_subscriber::fmt().init();
    let port: &str = "5000";

    tracing::info!("router initialized, now listening on port {}", port);

    let listener: tokio::net::TcpListener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .unwrap();

    let app: Router = Router::new()
        .route("/", get(root))
        .route("/leader_test", get(leader_test_get).post(leader_test_post));

    axum::serve(listener, app).await.unwrap();

    Ok(())
}


// Any filter defined in the module `filters` is accessible in your template.
mod filters {
    // This filter requires a `usize` input when called in templates
    pub fn replace<T: std::fmt::Display>(s: T, original: &str, replace: &str) -> ::askama::Result<String> {
        let s: String = s.to_string();
        Ok(s.replace(original, replace))
    }
}
