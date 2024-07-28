use std::{collections::HashMap, str::FromStr, sync::Arc};

use askama_axum::Template; // bring trait in scope
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
    Extension,
    Form,
    extract::State
};
use axum_extra::extract::CookieJar;
use serde::Deserialize;

use crate::{
    auth::{
        handlers::{login_user_handler, logout_handler, register_user_handler}, 
        middleware::{AuthError, AuthStatus},
        model::User
    },
    exam::models::{generate_leader_test, parse_test_form_data, save_test_to_database, BonusPointName, GradedBonusPoint, GradedPattern, GradedTechnique, PatternName, ScoringCategoryName, TechniqueName, TechniqueScoringHeaderName, Testee}, 
    filters, 
    AppState,
};

// #######################################################################################################################################################
// home.html
// #######################################################################################################################################################

#[derive(Template)]
#[template(path = "./primary_templates/home.html")] 
pub struct HomeTemplate {}

// Block rendering functionality is currently not implemented in Askama. Instead of using server-side partial rendering,
// I will just use hx-select to grab <div id="primary-content"> that is in my base template
// #[derive(Template)]
// #[template(path = "./primary_templates/home.html", block = "content")] 
// pub struct HomeTemplateContent {}

pub async fn get_home_page() -> impl IntoResponse  {
    let template: HomeTemplate = HomeTemplate {};

    (StatusCode::OK, Html(template.render().unwrap()))
}

// #######################################################################################################################################################
// user_dropdown.html
// #######################################################################################################################################################

#[derive(Template)]
#[template(path = "./partial_templates/user_dropdown.html")] 
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
#[template(path = "./auth_templates/sign-up.html")] 
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
            AuthError::DuplicateEmail => return (StatusCode::INTERNAL_SERVER_ERROR, Html("<h1>Duplicate Email</h1>")).into_response(),
            AuthError::InternalServerError(ee) => return (StatusCode::INTERNAL_SERVER_ERROR, Html(format!("{:?}", ee))).into_response(),
            _ => return (StatusCode::INTERNAL_SERVER_ERROR, Html("<h1>Unexpected error occurred, this should never happen.</h1>")).into_response() // This should never happen
        }
    }
}


// #######################################################################################################################################################
// login.html
// #######################################################################################################################################################


#[derive(Template)]
#[template(path = "./auth_templates/login.html")] 
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
            AuthError::InvalidEmailOrPassword => return (StatusCode::OK, Html("<h1>Invalid Email or Password</h1>")).into_response(),
            AuthError::InternalServerError(ee) => return (StatusCode::INTERNAL_SERVER_ERROR, Html(format!("{:?}", ee))).into_response(),
            _ => return (StatusCode::INTERNAL_SERVER_ERROR, Html("<h1>Unexpected error occurred</h1>")).into_response()
        }
    }
}

// #######################################################################################################################################################
// logout endpoint
// #######################################################################################################################################################

/// Logout the user and return them to the home page
pub async fn get_logout_page(
    cookie_jar: CookieJar,
    State(data): State<Arc<AppState>>,
    Extension(auth_status): Extension<AuthStatus>
) -> impl IntoResponse {

    // To get to this page requires auth so we can expect an authorized user variant of auth status instaed of autherror
    let authorized_user = match auth_status {
        AuthStatus::Authorized(user) => user,
        AuthStatus::Unauthorized(_) => panic!("If this happens, check your auth middleware application.")
    };

    match logout_handler(cookie_jar, authorized_user, data).await {
        Ok(response) => return response.into_response(),
        Err(e) => match e {
            AuthError::NotLoggedIn => return Redirect::to("/").into_response(),
            AuthError::InternalServerError(ee) => return (StatusCode::INTERNAL_SERVER_ERROR, Html(format!("{:?}", ee))).into_response(),
            _ => return (StatusCode::INTERNAL_SERVER_ERROR, Html("<h1>Unexpected error occurred</h1>")).into_response()
        }
    }
}

// #######################################################################################################################################################
// dashboard.html
// #######################################################################################################################################################

#[derive(Template)]
#[template(path = "./primary_templates/dashboard.html")] 
pub struct DashboardTemplate {}

pub async fn get_dashboard_page() -> impl IntoResponse  {
    let template: DashboardTemplate = DashboardTemplate {};

    (StatusCode::OK, Html(template.render().unwrap()))
}


// #######################################################################################################################################################
// leader_test.html
// #######################################################################################################################################################


pub async fn get_leader_test_page() -> impl IntoResponse  {
    let template = generate_leader_test();

    (StatusCode::OK, Html(template.render().unwrap()))
}

pub async fn post_leader_test_form(
    State(data): State<Arc<AppState>>,
    Form(test): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    let (pattern_scores, technique_scores, bonus_scores, testee) = parse_test_form_data(test);

    save_test_to_database(&data.db, &testee, pattern_scores, technique_scores, bonus_scores)
    // Print or process the parsed data
    println!("Pattern Scores: {:?}", pattern_scores);
    println!("Technique Scores: {:?}", technique_scores);
    println!("Bonus Scores: {:?}", bonus_scores);
    println!("Testee: {:?}", testee);

    StatusCode::OK
}
