use std::{collections::HashMap, default, sync::Arc};

use askama_axum::Template; // bring trait in scope
use axum::{
    extract::{Host, Path, Query, State}, http::{HeaderMap, StatusCode}, response::{Html, IntoResponse, Redirect}, Extension, Form, Json
};
use axum_extra::extract::CookieJar;
use chrono::NaiveDateTime;
use lettre::{transport::smtp::authentication::Credentials, AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::{
    auth::{
        handlers::{google_oauth_callback_handler, google_oauth_init_flow_handler, login_user_handler, logout_handler, register_user_handler, GoogleOAuthCallbackParams}, 
        middleware::{AuthError, AuthStatus},
        model::User
    }, exam::{
        handlers::{create_testee, dequeue_testee, enqueue_testee, fetch_test_results_by_id, fetch_testee_by_id, fetch_testee_tests_by_id, fetch_tests_by_status, fetch_unique_test_names, parse_test_form_data, retrieve_queue, save_test_to_database, search_for_testee, send_email, TestError}, 
        models::{FullTestSummary, Proctor, Test, TestGradeSummary, TestListItem, Testee}
    }, filters, AppState
};

/// A helper function to handle errors consistently
fn error_response(message: &str) -> impl IntoResponse {
    (StatusCode::OK, Html(format!("<h1 id=\"primary-content\">{}</h1>", message))).into_response()
}

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
    is_demo_mode: bool,
    google_oauth_enabled: bool,
}

pub async fn get_login_page(State(data): State<Arc<AppState>>) -> impl IntoResponse  {
    let template: LoginTemplate = LoginTemplate {is_demo_mode: data.env.is_demo_mode, google_oauth_enabled: data.google_oauth_config.is_some()};

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
    cookie_jar: CookieJar,
    State(data): State<Arc<AppState>>,
    Form(login) : Form<LoginForm>,
) -> impl IntoResponse {

    match login_user_handler(data, cookie_jar, login.email, login.password).await {
        Ok(response) => return response.into_response(),
        Err(e) => match e {
            AuthError::InvalidEmailOrPassword => return (StatusCode::OK, Html("<h1>Invalid Email or Password</h1>")).into_response(),
            AuthError::InternalServerError(ee) => return (StatusCode::OK, Html(format!("Error: {:?}", ee))).into_response(),
            _ => return (StatusCode::OK, Html("<h1>Error: Unexpected error occurred</h1>")).into_response()
        }
    }
}

pub async fn get_google_oauth_init_flow(
    State(data): State<Arc<AppState>>,
    cookie_jar: CookieJar,
) -> impl IntoResponse {

    match google_oauth_init_flow_handler(data, cookie_jar).await {
        Ok(response) => return response.into_response(),
        Err(e) => match e {
            AuthError::OAuthError(ee) => return (StatusCode::OK, Html(format!("OAuth Error: {:?}", ee))).into_response(),
            AuthError::InternalServerError(ee) => return (StatusCode::OK, Html(format!("Error: {:?}", ee))).into_response(),
            _ => return (StatusCode::OK, Html("<h1>Error: Unexpected error occurred</h1>")).into_response()
        }
    }
    
}

pub async fn get_google_oauth_callback(
    cookie_jar: CookieJar,
    State(data): State<Arc<AppState>>,
    Query(callback_params): Query<GoogleOAuthCallbackParams>,
) -> impl IntoResponse {

    match google_oauth_callback_handler(data, cookie_jar, callback_params).await {
        Ok(response) => return response.into_response(),
        Err(e) => match e {
            AuthError::OAuthError(ee) => return (StatusCode::OK, Html(format!("OAuth Error: {:?}", ee))).into_response(),
            AuthError::InternalServerError(ee) => return (StatusCode::OK, Html(format!("Error: {:?}", ee))).into_response(),
            AuthError::AccountNotFound => return (StatusCode::OK, Html("<h1>You do not yet have an account. Create an account on our sign-up page using your Google account's email address and in the future you will be able to sign in with Google.</h1>".to_string())).into_response(),
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
pub struct DashboardTemplate {
    test_names: Vec<String>
}

pub async fn get_dashboard_page(State(data): State<Arc<AppState>>) -> impl IntoResponse  {
    let test_names = data.test_configurations.tests.iter().map(|test| test.metadata.test_name.clone()).collect();
    let template: DashboardTemplate = DashboardTemplate {test_names};
    (StatusCode::OK, Html(template.render().unwrap()))
}


// #######################################################################################################################################################
// dancer_test.html
// #######################################################################################################################################################

#[derive(Template)]
#[template(path = "./primary_templates/dancer_test.html")] 
pub struct DancerTestPageTemplate {
    test: Test,
    prefilled_user_info: PrefilledTestData,
    test_summary: Option<FullTestSummary>,
    test_index: i32, // Used for on the fly test grading
    is_demo_mode: bool,
    email_functionality_active: bool,
}

#[derive(Deserialize)]
pub struct PrefilledTestData {
    first_name: Option<String>,
    last_name: Option<String>,
    email: Option<String>,
}

pub async fn get_test_page(
    State(data): State<Arc<AppState>>,
    Path(test_index): Path<i32>,
    Query(prefilled_user_info): Query<PrefilledTestData>,
) -> impl IntoResponse  {

    if let Some(test) = data.test_configurations.tests.get(test_index as usize) {
        let template = DancerTestPageTemplate {
            test: test.clone(),
            prefilled_user_info,
            test_summary: None,
            test_index,
            is_demo_mode: data.env.is_demo_mode,
            email_functionality_active: data.smtp_config.is_some()
        };
        (StatusCode::OK, Html(template.render().unwrap()))
        
    } else {
        (StatusCode::OK, Html(format!("<h1 id=\"primary-content\">Error: Invalid test index ({}) in url.</h1>", test_index)))
    }
}

/// Handles parsing the test form, saving the graded test to the database, and emailing test results to the testee.
pub async fn post_test_form(
    State(data): State<Arc<AppState>>,
    Extension(auth_status): Extension<AuthStatus>,
    Path(test_index): Path<i32>,
    Host(server_root_url): Host,
    Form(test): Form<HashMap<String, String>>,
) -> impl IntoResponse {

    let proctor = match auth_status {
        AuthStatus::Authorized(user) => Proctor { id: user.user.id, first_name: user.user.first_name, last_name: user.user.last_name},
        AuthStatus::Unauthorized(e) => return error_response(&format!("Unauthorized: {:?}", e)).into_response()
    };

    // By virtue of this existing, they want the email sent.
    let testee_wants_email_sent = test.get("send_email_results").is_some();

    if let Some(test_definition) = data.test_configurations.tests.get(test_index as usize) {
        match parse_test_form_data(test, test_definition.clone(), Some(proctor)) {
            Ok(graded_test) => {
                match save_test_to_database(&data.db, graded_test).await {
                    Ok(testee_id) => {
                        if let (
                            Some(smtp_config), 
                            Some(smtp_mailer), 
                            true) = (
                                data.smtp_config.clone(), 
                                data.smtp_mailer.clone(),
                                testee_wants_email_sent
                            ) {
                            tokio::spawn(async move {
                                if let Err(e) = send_email(&data.db, &smtp_mailer, smtp_config, testee_id, server_root_url).await {
                                    eprintln!("Failed to send email: {:?}", e);
                                }
                            });
                        };
                        Redirect::to("/dashboard").into_response()
                    },
                    Err(e) => error_response(&format!("Error saving test to database: {:?}", e)).into_response()
                }
            },
            Err(e) => error_response(&format!("Error parsing test form data: {:?}", e)).into_response()
        }
    } else {
        error_response(&format!("Invalid test index ({}) in URL", test_index)).into_response()
    }
}

// #######################################################################################################################################################
// test_grade.html
// #######################################################################################################################################################

#[derive(Template)]
#[template(path = "./partial_templates/test_grade.html")] 
pub struct GradeTestTemplate {
    grade_summary: TestGradeSummary,
    test_date: Option<NaiveDateTime>,
    proctor_first_name: Option<String>,
    proctor_last_name: Option<String>,
}

/// Used to grade a test on the fly.
pub async fn post_grade_test(
    State(data): State<Arc<AppState>>,
    Path(test_index): Path<i32>,
    Form(test): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    if let Some(test_definition) = data.test_configurations.tests.get(test_index as usize) {
        match parse_test_form_data(test, test_definition.clone(), None) {
            Ok(mut parsed_test) => {
                
                match parsed_test.grade() {
                    Ok(_) => (),
                    Err(e) => return error_response(&format!("Error grading test in post_grade_test function: {:?}", e)).into_response()
                };
                
                let grade_summary = match parsed_test.grade_summary() {
                    Ok(summary) => summary,
                    Err(e) => return error_response(&format!("Error summarizing test in post_grade_test function: {:?}", e)).into_response()
                };

                let template = GradeTestTemplate {
                    grade_summary,
                     // Feed in None for the following stuff because we don't need it when administering a test
                    // Since this function is used to grade a test on the fly
                    test_date: None,
                    proctor_first_name: None,
                    proctor_last_name: None
                };

                return (StatusCode::OK, Html(template.render().unwrap())).into_response()
            },
            Err(e) => return error_response(&format!("Error parsing test form data: {:?}", e)).into_response()
        }
    } else {
        return error_response(&format!("Invalid test index ({}) in URL", test_index)).into_response()
    };
}



// // #######################################################################################################################################################
// // Json Test Results API
// // #######################################################################################################################################################

// pub async fn get_json_test_results(
//     State(data): State<Arc<AppState>>,
//     Path(test_id): Path<i32>,
// ) -> impl IntoResponse {
//     match fetch_test_results_by_id(&data.db, test_id).await {
//         Ok(test_result) => match test_result {
//             Some(graded_test) => (StatusCode::OK, Json(graded_test)).into_response(),
//             None => (StatusCode::NOT_FOUND, Json("No test with that ID found")).into_response(),
//         }
//         Err(TestError::InternalServerError(err)) => {
//             (StatusCode::OK, Json(format!("Error: {:?}", err))).into_response()
//         }
//     }
// }

// #######################################################################################################################################################
// dancer_test.html
// #######################################################################################################################################################

#[derive(Template)]
#[template(path = "./primary_templates/dancer_test.html")] 
pub struct GradedTestTemplate {
    test: Test,
    test_summary: Option<FullTestSummary>,
    test_index: i32, // Unused for this template
    prefilled_user_info: PrefilledTestData,
    is_demo_mode: bool,
    email_functionality_active: bool, // Unused for this template
}

pub async fn get_test_results(
    State(data): State<Arc<AppState>>,
    Path(test_id): Path<Uuid>,
) -> impl IntoResponse {
    match fetch_test_results_by_id(&data.db, test_id).await {
        Ok(Some(test)) => {
            let prefilled_user_info = PrefilledTestData{
                first_name: Some(test.metadata.testee.clone().expect("Invariant that graded tests all have Testees violated in get_test_results fn").first_name),
                last_name: Some(test.metadata.testee.clone().expect("Invariant that graded tests all have Testees violated in get_test_results fn").last_name),
                email: Some(test.metadata.testee.clone().expect("Invariant that graded tests all have Testees violated in get_test_results fn").email)
            };

            let test_summary = match test.full_summary() {
                Ok(summary) => Some(summary),
                Err(e) => return error_response(&format!("Error summarizing test in get_test_results function: {:?}", e)).into_response()
            };

            let template = GradedTestTemplate {
                test,
                prefilled_user_info,
                test_index: -1,
                test_summary,
                is_demo_mode: data.env.is_demo_mode,
                email_functionality_active: false,
            };
            match template.render() {
                Ok(rendered) => Html(rendered).into_response(),
                Err(e) => {
                    (StatusCode::OK, Html(format!("<h1 id=\"primary-content\">Error: {:?}</h1>", e))).into_response()
                }
            }
        },
        Ok(None) => error_response(&format!("No test found for test id ({}) in URL", test_id)).into_response(),
        Err(TestError::InternalServerError(err)) => {
            (StatusCode::OK, Html(format!("<h1 id=\"primary-content\">Error: {:?}<h1>", err))).into_response()
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
    option_test_summaries: Option<Vec<FullTestSummary>>,
    option_testee: Option<Testee>,
}

pub async fn get_test_summaries(
    State(data): State<Arc<AppState>>,
    Path(testee_id): Path<Uuid>,
) -> impl IntoResponse {

    let option_test_summaries = match fetch_testee_tests_by_id(&data.db, testee_id).await {
        Ok(option) => option,
        Err(e) => match e {
            TestError::InternalServerError(msg) => return (StatusCode::OK, Html(format!("<h1 id=\"primary-content\">Error: {:?}</h1>", msg))).into_response(),
            _ => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Undefined behavior. This should never happen." }))).into_response()
        }
    };

    let option_testee = match fetch_testee_by_id(&data.db, testee_id).await {
        Ok(option) => option,
        Err(e) => match e {
            TestError::InternalServerError(msg) => return (StatusCode::OK, Html(format!("<h1 id=\"primary-content\">Error: {:?}</h1>", msg))).into_response(),
            _ => return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Undefined behavior. This should never happen." }))).into_response()
        }
    };

    let template = TestSummariesTemplate {
        option_test_summaries,
        option_testee,
    };
    (StatusCode::OK, Html(template.render().unwrap())).into_response()
}

// #######################################################################################################################################################
// broad_test_results.html
// #######################################################################################################################################################

#[derive(Template)]
#[template(path = "./primary_templates/broad_test_results.html")] 
pub struct BroadTestResultsTemplate {
    test_names: Vec<String>,
    test_list_items: Option<Vec<TestListItem>>,
}

#[derive(Deserialize)]
pub struct TestFilterQuery {
    #[serde(default)]
    test_names: Vec<String>,

    pass_filter: Option<String>,
}

pub async fn get_broad_test_results(
    State(data): State<Arc<AppState>>,
    axum_extra::extract::Query(form_data): axum_extra::extract::Query<TestFilterQuery>,
) -> impl IntoResponse {
    
    let mut test_names = match fetch_unique_test_names(&data.db).await {
        Ok(names) => names,
        Err(e) => return error_response(&e.to_string()).into_response()
    };
    test_names.sort();

    
    // Convert this to an option or bool because the form returns strings and we need a bool
    let is_passing_filter = match form_data.pass_filter {
        Some(data) => match data.to_lowercase().as_str() {
            "passing" => Some(true),
            "failing" => Some(false),
            "both" => None,
            _ => return error_response("An unexpected form value was submitted. You're trying to mess with the website.").into_response()
        },
        None => None
    };

    let test_list_items = match form_data.test_names.is_empty() {
        false => match fetch_tests_by_status(&data.db, &form_data.test_names, is_passing_filter).await {
            Ok(vec) => Some(vec),
            Err(e) => return error_response(&e.to_string()).into_response()
        },
        true => None,
    };

    let template = BroadTestResultsTemplate {
        test_names: test_names,
        test_list_items
    };

    (StatusCode::OK, Html(template.render().unwrap())).into_response()
}


// #######################################################################################################################################################
// queue.html
// #######################################################################################################################################################

#[derive(Template)]
#[template(path = "./primary_templates/queue.html")] 
pub struct QueueTemplate {
    admin_user: bool,
    signup_key_required: bool,
    test_names: Vec<String>,
    queue: Vec<(Testee, usize)>,
    is_demo_mode: bool,
}

pub async fn get_queue(
    State(data): State<Arc<AppState>>,
    Extension(auth_status): Extension<AuthStatus>,
) -> impl IntoResponse {
    
    let admin_user = match auth_status {
        AuthStatus::Authorized(_) => true,
        AuthStatus::Unauthorized(_) => false
    };
 

    let queue = match retrieve_queue(&data.db).await {
        Ok(q) => q.into_iter()
            .map(|(testee, index)| (testee, index as usize))  // Convert i32 to usize
            .collect(),
        Err(e) => {
            return (StatusCode::OK, Html(format!("<h1 id=\"primary-content\">Error: {:?}</h1>", e))).into_response()
        }
    };
    let test_names = data.test_configurations.tests
        .iter()
        .map(|test| test.metadata.test_name.clone())
        .collect::<Vec<String>>();

    let template = QueueTemplate {
        admin_user,
        signup_key_required: (data.env.queue_signup_key != ""),
        queue,
        test_names,
        is_demo_mode: data.env.is_demo_mode
    };

    (StatusCode::OK, Html(template.render().unwrap())).into_response()
}

#[derive(Deserialize)]
pub struct EnqueueForm {
    first_name: String,
    last_name: String,
    email: String,
    signup_key: Option<String>,
    test_definition_index: i32,
}

pub async fn post_queue(
    State(data): State<Arc<AppState>>,
    Form(user_info): Form<EnqueueForm>,
) -> impl IntoResponse {

    let signup_key_required = data.env.queue_signup_key != "";
    
    if signup_key_required {
        if user_info.signup_key.unwrap_or_default() != data.env.queue_signup_key {
            return error_response("Invalid sign-up key. Refresh the page and try again.").into_response()
        }
    }

    let testee = match create_testee(
        &data.db, user_info.first_name.as_str(), user_info.last_name.as_str(), user_info.email.as_str()
    ).await {
        Ok(person) => person,
        Err(e) => {
            return (StatusCode::OK, Html(format!("<h1 id=\"primary-content\">Error: {:?}</h1>", e))).into_response()
        }
    };

    // Create testee 100% returns a testee with a testee id, so I can call unwrap on this
    if let Err(e) = enqueue_testee(&data.db, testee.id.unwrap(), user_info.test_definition_index).await {
        return (StatusCode::OK, Html(format!("<h1 id=\"primary-content\">Error enqueuing testee: {:?}</h1>", e))).into_response();
    }
    
    Redirect::to("/queue").into_response()
}

// #######################################################################################################################################################
// dequeue
// #######################################################################################################################################################

#[derive(Deserialize, Debug)]
pub struct DequeueParams {
    testee_id: Option<Uuid>,
    test_definition_index: Option<i32>,
}

/// Removes a user from the queue upon receiving a delete request. If called with a request header HX-Trigger equal to 
/// "administer-test-button", will redirect to the proper administer test page with the query parameters
/// equal to the queue user's information. If there is no response header, just deletes the user and returns empty html.
pub async fn delete_dequeue(
    State(data): State<Arc<AppState>>,
    Query(params): Query<DequeueParams>,
    headers: HeaderMap,
) -> impl IntoResponse {

    let (testee, test_definition_index) = match dequeue_testee(&data.db, params.testee_id, params.test_definition_index).await {
        Ok(option) => match option {
            Some(result) => (result.0, result.1),
            None => return (StatusCode::OK, Html("<h1 id=\"primary-content\">Error: No testee with that ID found --> Perhaps the queue was empty.</h1>")).into_response(),
        },
        Err(e) => {
            return (StatusCode::OK, Html(format!("<h1 id=\"primary-content\">Error dequeuing testee: {:?}</h1>", e))).into_response();
        }
    };

    if let Some(header_value) = headers.get("HX-Trigger") {
        if header_value == "administer-test-button" {
            // Convert the role and testee fields to strings for use in the URL
            let first_name = testee.first_name.to_string();
            let last_name = testee.last_name.to_string();
            let email = testee.email.to_string();

            let redirect_url = format!(
                "/administer-test/{}?first_name={}&last_name={}&email={}",
                test_definition_index, first_name, last_name, email
            );

            return Redirect::to(&redirect_url).into_response();
        }
    }

    (StatusCode::OK, Html("")).into_response()
}

// #######################################################################################################################################################
// get send email
// #######################################################################################################################################################


// /// This is intended to enable you to manually force the server to send someone an email with links to their results.
// pub async fn post_send_email(State(data): State<Arc<AppState>>) -> impl IntoResponse {
    
//     let smtp_config = match &data.smtp_config {
//         Some(config) => config,
//         None => return (StatusCode::OK, error_response("SMTP not set up. No email functionality available."))
//     };

//     // Create the email message
//     let email = Message::builder()
//         .from(smtp_config.user_email.parse().unwrap())
//         .to("paulrcreamer@gmail.com".parse().unwrap()) // Change this to the actual recipient's email
//         .subject("Test Email")
//         .body("This is a test email sent using Lettre. Paul if you see this, this means the email functionality is coming together".to_string())
//         .unwrap();

//     // Set up the SMTP transport
//     let creds = Credentials::new(
//         smtp_config.user_login.clone(),
//         smtp_config.user_password.clone(),
//     );

//     let mailer: AsyncSmtpTransport<Tokio1Executor> = AsyncSmtpTransport::<Tokio1Executor>::relay(&smtp_config.server_host)
//         .unwrap()
//         .credentials(creds)
//         .build();

//     // Send the email
//     match mailer.send(email).await {
//         Ok(_) => {
//             println!("Email sent successfully!");
//             return (StatusCode::OK, error_response("Email sent."))
//         }
//         Err(e) => {
//             eprintln!("Failed to send email: {:?}", e);
//             return (StatusCode::OK, error_response("Email not sent."))
//         }
//     }
// }