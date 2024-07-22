use std::sync::Arc;

use askama_axum::Template; // bring trait in scope
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
    Extension,
    Form,
    extract::State
};
use jsonwebtoken::crypto::sign;
use serde::Deserialize;

use crate::{
    auth::{
        handlers::{login_user_handler, register_user_handler, LoginError, SignInSignUpError}, middleware::AuthStatus, model::User
    },
    AppState,
};

// #######################################################################################################################################################
// root.html
// #######################################################################################################################################################



#[derive(Template)]
#[template(path = "home.html")] 
pub struct RootTemplate {}

pub async fn get_root_page() -> impl IntoResponse  {
    let template: RootTemplate = RootTemplate {};

    (StatusCode::OK, Html(template.render().unwrap()))
}

// #######################################################################################################################################################
// user_dropdown.html
// #######################################################################################################################################################

#[derive(Template)]
#[template(path = "user_dropdown.html")] 
pub struct UserDropdownTemplate {
    user: Option<User>
}

pub async fn get_user_dropdown(
    Extension(auth_status): Extension<AuthStatus>,
) -> impl IntoResponse {
    
    let user = match auth_status {
        AuthStatus::Authorized(authorized_user) => Some(authorized_user.user),
        AuthStatus::Unauthorized(_) => None
    };

    let template = UserDropdownTemplate { user };

    (StatusCode::OK, Html(template.render().unwrap()))
}

// #######################################################################################################################################################
// sign-up.html
// #######################################################################################################################################################

#[derive(Template)]
#[template(path = "sign-up.html")] 
pub struct SignUpTemplate {
}

pub async fn get_signup_page() -> impl IntoResponse {
    let template = SignUpTemplate {};

    (StatusCode::OK, Html(template.render().unwrap()))
}

#[derive(Debug, Deserialize)]
pub struct SignUpForm {
    first_name: String,
    last_name: String,
    email: String,
    password: String,
    confirm_password: String,
}

pub async fn post_signup_form(
    State(data): State<Arc<AppState>>,
    Form(sign_up) : Form<SignUpForm>,
) -> impl IntoResponse {
    println!("{:?}", sign_up);

    // Validate form data
    if sign_up.password != sign_up.confirm_password {
        return (
            StatusCode::BAD_REQUEST,
            Html("<h1>Passwords do not match</h1>"),
        ).into_response();
    }

    let user_registered = register_user_handler(data, sign_up.first_name, sign_up.last_name, sign_up.email, sign_up.password).await;

    match user_registered {
        Ok(_) => return Redirect::to("/login").into_response(),
        Err(e) => match e {
            SignInSignUpError::DuplicateEmail => return (StatusCode::OK, Html("<h1>Duplicate Email</h1>")).into_response(),
            SignInSignUpError::InternalServerError(ee) => return (StatusCode::OK, Html(format!("{:?}", ee))).into_response()
        }
    }
}


// #######################################################################################################################################################
// login.html
// #######################################################################################################################################################


#[derive(Template)]
#[template(path = "login.html")] 
pub struct LoginTemplate {}

pub async fn get_login_page() -> impl IntoResponse  {
    let template: LoginTemplate = LoginTemplate {};

    (StatusCode::OK, Html(template.render().unwrap()))
}

#[derive(Debug, Deserialize)]
pub struct LoginForm {
    email: String,
    password: String,
}

pub async fn post_login_form(
    State(data): State<Arc<AppState>>,
    Form(login) : Form<LoginForm>,
) -> impl IntoResponse {
    println!("{:?}", login);

    match login_user_handler(data, login.email, login.password).await {
        Ok(response) => return response.into_response(),
        Err(e) => match e {
            LoginError::InvalidEmailOrPassword => return (StatusCode::OK, Html("<h1>Invalid Email or Password</h1>")).into_response(),
            LoginError::InternalServerError(ee) => return (StatusCode::OK, Html(format!("{:?}", ee))).into_response()
        }
    }
}

// #######################################################################################################################################################
// dashboard.html
// #######################################################################################################################################################

#[derive(Template)]
#[template(path = "dashboard.html")] 
pub struct DashboardTemplate {}

pub async fn get_dashboard_page() -> impl IntoResponse  {
    let template: DashboardTemplate = DashboardTemplate {};

    (StatusCode::OK, Html(template.render().unwrap()))
}