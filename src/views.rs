use std::{collections::HashMap, sync::Arc};

use askama_axum::Template; // bring trait in scope
use axum::{
    extract::{Path, State, Query}, http::StatusCode, response::{Html, IntoResponse, Redirect}, Extension, Form, Json
};
use axum_extra::extract::CookieJar;
use serde::Deserialize;
use serde_json::json;

use crate::{
    auth::{
        handlers::{login_user_handler, logout_handler, register_user_handler}, 
        middleware::{AuthError, AuthStatus},
        model::User
    },
    exam::{
        handlers::{fetch_test_results_by_id, fetch_testee_by_id, fetch_testee_tests_by_id, parse_test_form_data, save_test_to_database, search_for_testee, TestError}, models::{generate_follower_test, generate_leader_test, GradedTest, TestSummary, TestType, Testee}
    },
    AppState,
    filters,
};

// #######################################################################################################################################################
// home.html
// #######################################################################################################################################################

#[derive(Template)]
#[template(path = "./primary_templates/home.html")] 
pub struct HomeTemplate { is_demo_mode: bool }

// Block rendering functionality is currently not implemented in Askama. Instead of using server-side partial rendering,
// I will just use hx-select to grab <div id="primary-content"> that is in my base template
// #[derive(Template)]
// #[template(path = "./primary_templates/home.html", block = "content")] 
// pub struct HomeTemplateContent {}

pub async fn get_home_page(    State(data): State<Arc<AppState>>) -> impl IntoResponse  {
    let template: HomeTemplate = HomeTemplate { is_demo_mode: data.env.is_demo_mode };

    (StatusCode::OK, Html(template.render().unwrap()))
}

// #######################################################################################################################################################
// contact.html
// #######################################################################################################################################################

#[derive(Template)]
#[template(path = "./primary_templates/contact.html")] 
pub struct ContactTemplate {}

pub async fn get_contact_page() -> impl IntoResponse  {
    let template: ContactTemplate = ContactTemplate {};

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
    licensing_key: String,
}

/// All the errors must return the OK status code for HTMX. Also, they must have an outer element with an id of primary-content
pub async fn post_signup_form(
    State(data): State<Arc<AppState>>,
    Form(sign_up) : Form<SignUpForm>,
) -> impl IntoResponse {

    // Validate form data
    if sign_up.password != sign_up.confirm_password {
        return (
            StatusCode::OK,
            Html("<h1 id=\"primary-content\">Error: Passwords do not match</h1>"),
        ).into_response();
    }

    let user_registered = register_user_handler(data, sign_up.first_name, sign_up.last_name, sign_up.email, sign_up.password, sign_up.licensing_key).await;

    match user_registered {
        Ok(_) => return Redirect::to("/login").into_response(),
        Err(e) => match e {
            AuthError::DuplicateEmail => return (StatusCode::OK, Html("<h1 id=\"primary-content\">Error: Duplicate Email</h1>")).into_response(),
            AuthError::InvalidLicensingKey => return (StatusCode::OK, Html("<h1 id=\"primary-content\">Error: Invalid Licensing Key</h1>")).into_response(),
            AuthError::InternalServerError(ee) => return (StatusCode::OK, Html(format!("<h1 id=\"primary-content\">Error: {:?}</h1>", ee))).into_response(),
            _ => return (StatusCode::INTERNAL_SERVER_ERROR, Html("<h1 id=\"primary-content\">Unexpected error occurred, this should be impossible.</h1>")).into_response() // This should never happen
        }
    }
}


// #######################################################################################################################################################
// login.html
// #######################################################################################################################################################


#[derive(Template)]
#[template(path = "./auth_templates/login.html")] 
pub struct LoginTemplate {
    is_demo_mode: bool
}

pub async fn get_login_page(State(data): State<Arc<AppState>>) -> impl IntoResponse  {
    let template: LoginTemplate = LoginTemplate {is_demo_mode: data.env.is_demo_mode};

    (StatusCode::OK, Html(template.render().unwrap()))
}

#[derive(Debug, Deserialize)]
pub struct LoginForm {
    email: String,
    password: String,
}

/// Login form doesn't use HTMX to force reload of the navbar (to get the user in the top right)
/// so the html can return error status codes and the id of the outer element does not matter (unlike signup)
pub async fn post_login_form(
    State(data): State<Arc<AppState>>,
    Form(login) : Form<LoginForm>,
) -> impl IntoResponse {
    println!("{:?}", login);

    match login_user_handler(data, login.email, login.password).await {
        Ok(response) => return response.into_response(),
        Err(e) => match e {
            AuthError::InvalidEmailOrPassword => return (StatusCode::OK, Html("<h1>Invalid Email or Password</h1>")).into_response(),
            AuthError::InternalServerError(ee) => return (StatusCode::OK, Html(format!("Error: {:?}", ee))).into_response(),
            _ => return (StatusCode::OK, Html("<h1>Error: Unexpected error occurred</h1>")).into_response()
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
            AuthError::InternalServerError(ee) => return (StatusCode::OK, Html(format!("Error: {:?}", ee))).into_response(),
            _ => return (StatusCode::OK, Html("<h1>Error: Unexpected error occurred</h1>")).into_response()
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


pub async fn get_leader_test_page(State(data): State<Arc<AppState>>) -> impl IntoResponse  {
    let template = generate_leader_test(data.env.is_demo_mode);

    (StatusCode::OK, Html(template.render().unwrap()))
}

pub async fn post_leader_test_form(
    State(data): State<Arc<AppState>>,
    Form(test): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    let graded_test = parse_test_form_data(test, TestType::Leader, generate_leader_test(data.env.is_demo_mode));

    match save_test_to_database(&data.db, graded_test).await {
        Ok(_) => Redirect::to("/dashboard").into_response(),
        Err(e) => return (StatusCode::OK, Html(format!("Error: {:?}", e))).into_response(),
    }
}

// #######################################################################################################################################################
// follower_test.html
// #######################################################################################################################################################

/// This could've been refactored to avoid copy-pasting the leader functions, but tbh this is a spot where it wasn't worth the effort
pub async fn get_follower_test_page(State(data): State<Arc<AppState>>) -> impl IntoResponse  {
    let template = generate_follower_test(data.env.is_demo_mode);

    (StatusCode::OK, Html(template.render().unwrap()))
}

pub async fn post_follower_test_form(
    State(data): State<Arc<AppState>>,
    Form(test): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    let graded_test = parse_test_form_data(test, TestType::Follower, generate_follower_test(data.env.is_demo_mode));

    match save_test_to_database(&data.db, graded_test).await {
        Ok(_) => Redirect::to("/dashboard").into_response(),
        Err(e) => return (StatusCode::OK, Html(format!("Error: {:?}", e))).into_response(),
    }
}

// #######################################################################################################################################################
// test_grade.html
// #######################################################################################################################################################

#[derive(Template)]
#[template(path = "./partial_templates/test_grade.html")] 
pub struct GradeTestTemplate {
    score: u32,
    passing_score: u32,
    max_score: u32,
}

pub async fn post_grade_test(
    State(data): State<Arc<AppState>>,
    Form(test): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    let graded_test = parse_test_form_data(test, TestType::Leader, generate_leader_test(data.env.is_demo_mode));

    let template = GradeTestTemplate {
        score: graded_test.score,
        passing_score: graded_test.passing_score,
        max_score: graded_test.max_score
    };
    
    (StatusCode::OK, Html(template.render().unwrap()))
}



// #######################################################################################################################################################
// Json Test Results API
// #######################################################################################################################################################

pub async fn get_json_test_results(
    State(data): State<Arc<AppState>>,
    Path(test_id): Path<i32>,
) -> impl IntoResponse {
    match fetch_test_results_by_id(&data.db, test_id).await {
        Ok(test_result) => match test_result {
            Some(graded_test) => (StatusCode::OK, Json(graded_test)).into_response(),
            None => (StatusCode::NOT_FOUND, Json("No test with that ID found")).into_response(),
        }
        Err(TestError::InternalServerError(err)) => {
            (StatusCode::OK, Html(format!("Error: {:?}", err))).into_response()
        }
    }
}

// #######################################################################################################################################################
// request_test.html
// #######################################################################################################################################################

#[derive(Template)]
#[template(path = "./primary_templates/graded_test.html")] 
pub struct GradedTestTemplate {
    test: Option<GradedTest>,
}

pub async fn get_test_results(
    State(data): State<Arc<AppState>>,
    Path(test_id): Path<i32>,
) -> impl IntoResponse {
    match fetch_test_results_by_id(&data.db, test_id).await {
        Ok(test_result) => {
            let template = GradedTestTemplate {
                test: test_result,
            };
            match template.render() {
                Ok(rendered) => Html(rendered).into_response(),
                Err(e) => {
                    (StatusCode::OK, Html(format!("Error: {:?}", e))).into_response()
                }
            }
        }
        Err(TestError::InternalServerError(err)) => {
            (StatusCode::OK, Html(format!("Error: {:?}", err))).into_response()
        }
    }
}

// #######################################################################################################################################################
// search_testee.html
// #######################################################################################################################################################

#[derive(Template)]
#[template(path = "./primary_templates/search_testee.html")] 
pub struct SearchTesteeTemplate {
    is_demo_mode: bool,
    search_results: Option<Vec<Testee>>,
}

#[derive(Deserialize)]
pub struct SearchQuery {
    query: Option<String>,
}

pub async fn get_search_testee_form(
    State(data): State<Arc<AppState>>,
    Query(search_query): Query<SearchQuery>,
) -> impl IntoResponse {

    let search_results = if let Some(query) = search_query.query {
        search_for_testee(query, &data.db).await.ok().flatten()
    } else {
        None
    };

    let template = SearchTesteeTemplate { 
        is_demo_mode: data.env.is_demo_mode, 
        search_results: search_results
    };
    (StatusCode::OK, Html(template.render().unwrap())).into_response()
}

// #######################################################################################################################################################
// testee_test_summaries.html
// #######################################################################################################################################################

#[derive(Template)]
#[template(path = "./primary_templates/testee_test_summaries.html")] 
pub struct TestSummariesTemplate {
    option_test_summaries: Option<Vec<TestSummary>>,
    option_testee: Option<Testee>,
}

pub async fn get_test_summaries(
    State(data): State<Arc<AppState>>,
    Path(testee_id): Path<i32>,
) -> impl IntoResponse {

    let option_test_summaries = match fetch_testee_tests_by_id(&data.db, testee_id).await {
        Ok(option) => option,
        Err(e) => match e {
            TestError::InternalServerError(msg) => return (StatusCode::OK, Html(format!("Error: {:?}", msg))).into_response(),
            _ => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Undefined behavior. This should never happen." }))).into_response()
        }
    };

    let option_testee = match fetch_testee_by_id(&data.db, testee_id).await {
        Ok(option) => option,
        Err(e) => match e {
            TestError::InternalServerError(msg) => return (StatusCode::OK, Html(format!("Error: {:?}", msg))).into_response(),
            _ => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Undefined behavior. This should never happen." }))).into_response()
        }
    };

    let template = TestSummariesTemplate {
        option_test_summaries,
        option_testee,
    };
    (StatusCode::OK, Html(template.render().unwrap())).into_response()
}