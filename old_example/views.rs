use askama_axum::Template; // bring trait in scope
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
    Extension,
    routing::get, 
    Router,
    extract::Form,
};
use serde::Deserialize;

use crate::response::FilteredUser;
use crate::handler::filter_user_record;
use crate::jwt_auth::JWTAuthMiddleware;

// #######################################################################################################################################################
// root.html
// #######################################################################################################################################################

#[derive(Template)] // this will generate the code...
#[template(path = "root.html")] // using the template in this path, relative
                                 // to the `templates` dir in the crate root
pub struct RootTemplate {}

pub async fn get_root() -> impl IntoResponse  {
    let template: RootTemplate = RootTemplate {};

    (StatusCode::OK, Html(template.render().unwrap()))
}

// #######################################################################################################################################################
// user_or_sign_in.html
// #######################################################################################################################################################

/// This is used to display either the user information or a sign-in button on the nav bar
#[derive(Template)] 
#[template(path = "user_or_sign_in.html")] 
pub struct UserOrSignInTemplate {
    user: Option<FilteredUser>
}

pub async fn get_user_or_sign_in(
    Extension(jwtauth): Extension<JWTAuthMiddleware>,
) -> impl IntoResponse {
    // Attempt to extract the user information
    let user: Option<FilteredUser> = match filter_user_record(&jwtauth.user) {
        user => Some(user),
        None => None,
    };

    let template = UserOrSignInTemplate {
        user,
    };

    // Render the template and return the response
    (StatusCode::OK, Html(template.render().unwrap()))
}

// #######################################################################################################################################################
// leader_test.html
// #######################################################################################################################################################