use std::sync::Arc;

use axum::{
    middleware,
    routing::get,
    Router,
};

use crate::{
    auth::middleware::{check_auth_middleware, require_auth_middleware}, 
    views::{
        get_dashboard_page, 
        get_home_page,
        get_leader_test_page, 
        get_follower_test_page,
        get_login_page,
        get_logout_page,
        get_signup_page,
        get_user_dropdown,
        post_leader_test_form,
        post_follower_test_form,
        post_login_form, 
        post_signup_form
        },
    AppState,
};



pub fn create_router(app_state: Arc<AppState>) -> Router {
    Router::new()
    .route("/dashboard", get(get_dashboard_page))
    .route("/logout", get(get_logout_page))
    .route("/leader-test", get(get_leader_test_page).post(post_leader_test_form))
    .route("/follower-test", get(get_follower_test_page).post(post_follower_test_form))
        .route_layer(middleware::from_fn_with_state(app_state.clone(), require_auth_middleware))
    .route("/", get(get_home_page))
    .route("/sign-up", get(get_signup_page).post(post_signup_form))
    .route("/login", get(get_login_page).post(post_login_form))
    .route("/private/user-dropdown", get(get_user_dropdown)) 
        .route_layer(middleware::from_fn_with_state(app_state.clone(), check_auth_middleware))
    .with_state(app_state)
}