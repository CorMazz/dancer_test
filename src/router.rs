use std::sync::Arc;

use axum::{
    middleware,
    routing::{delete, get, post},
    Router,
};

use crate::{
    auth::middleware::{check_auth_middleware, require_auth_middleware}, 
    views::{
        delete_dequeue, get_broad_test_results, get_contact_page, get_dashboard_page, get_google_oauth_callback, get_google_oauth_init_flow, get_home_page, get_login_page, get_logout_page, get_queue, get_search_testee_form, get_signup_page, get_test_page, get_test_results, get_test_summaries, get_user_dropdown, post_grade_test, post_login_form, post_queue, post_signup_form, post_test_form
    },
    AppState
};

use tower_http::services::ServeDir;


pub fn create_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/dashboard", get(get_dashboard_page))
        .route("/logout", get(get_logout_page))
        .route("/administer-test/:test_index", get(get_test_page).post(post_test_form))
        .route("/private/grade-test/:test_index", post(post_grade_test))
        // .route("/api/v1/test-results/:test_id", get(get_json_test_results))
        .route("/search-testee", get(get_search_testee_form))
        .route("/test-summaries/:testee_id", get(get_test_summaries))
        .route("/queue/dequeue", delete(delete_dequeue))
        .route("/broad-test-results", get(get_broad_test_results))
        
    .route_layer(middleware::from_fn_with_state(app_state.clone(), require_auth_middleware))
    // Anything above this line will redirect to the login page if the user is not logged in

        .route("/", get(get_home_page))
        .route("/contact", get(get_contact_page))
        .route("/sign-up", get(get_signup_page).post(post_signup_form))
        .route("/login", get(get_login_page).post(post_login_form))
        .route("/queue", get(get_queue).post(post_queue))
        .route("/private/user-dropdown", get(get_user_dropdown)) 
        .route("/test-results/:test_id", get(get_test_results))
    .route_layer(middleware::from_fn_with_state(app_state.clone(), check_auth_middleware))
    // Anything above this line checks if the user is logged in and adds an AuthStatus extension to the request

    .route("/auth/google", get(get_google_oauth_init_flow))
    .route("/auth/google/callback", get(get_google_oauth_callback))


    .with_state(app_state)

    .nest_service("/static", ServeDir::new("static/"))
}