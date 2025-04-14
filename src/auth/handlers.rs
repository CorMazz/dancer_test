use std::sync::Arc;

use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use askama_axum::Response;
use axum::{
    http::{header, HeaderMap},
    response::{IntoResponse, Redirect},
};
use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    CookieJar,
};
use oauth2::{basic::BasicClient, AuthorizationCode, CsrfToken, PkceCodeChallenge, PkceCodeVerifier, Scope, TokenResponse};
use rand_core::OsRng;
use serde::Deserialize;
use sqlx::{Pool, Postgres};

use crate::{
    auth::{
        model::User,
        token::{TokenDetails, generate_jwt_token, verify_jwt_token},
        middleware::{AuthorizedUser, AuthError}
    },
    AppState,
};

use redis::{AsyncCommands, RedisError};

// #######################################################################################################################################################
// Sign Up
// #######################################################################################################################################################

/// Registers a user to the database
pub async fn register_user_handler(
    data: Arc<AppState>,
    first_name: String,
    last_name: String,
    email: String,
    password: String,
    licensing_key: String,
) -> Result<(), AuthError> {

    if licensing_key != data.env.signup_licensing_key {
        return Err(AuthError::InvalidLicensingKey)
    }

    if get_user(&email, &data.db).await?.is_some() {
        return Err(AuthError::DuplicateEmail);
    }

    let salt = SaltString::generate(&mut OsRng);
    let hashed_password = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AuthError::InternalServerError(Some(format!("Error while hashing password: {}", e))))
        .map(|hash| hash.to_string())?;

    sqlx::query_as!(
        User,
        "INSERT INTO users (first_name,last_name,email,password) VALUES ($1, $2, $3, $4)",
        first_name,
        last_name,
        email.to_ascii_lowercase(),
        hashed_password
    )
    .execute(&data.db)
    .await
    .map_err(|e| AuthError::InternalServerError(Some(format!("Database error: {}", e))))?;
    Ok(())
}


// #######################################################################################################################################################
// Login
// #######################################################################################################################################################


pub async fn login_user_handler(
    data: Arc<AppState>,
    cookie_jar: CookieJar,
    email: String,
    password: String,
) -> Result<impl IntoResponse, AuthError> {

    let user = get_user(&email, &data.db)
        .await?
        .ok_or_else(|| AuthError::InvalidEmailOrPassword)?;
    
    let is_valid = match PasswordHash::new(&user.password) {
        Ok(parsed_hash) => Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_or(false, |_| true),
        Err(_) => false,
    };

    if !is_valid {
        return Err(AuthError::InvalidEmailOrPassword);
    }

    let jar = login_user(user, &data, cookie_jar).await?;

    Ok((jar, Redirect::to("/dashboard")))
}

// #######################################################################################################################################################
// Google OAuth Initialize Login Flow Handler
// #######################################################################################################################################################

pub async fn google_oauth_init_flow_handler(
    data: Arc<AppState>,
    cookie_jar: CookieJar,
) -> Result<impl IntoResponse, AuthError> {
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    if let Some(config) = &data.google_oauth_config {
        let client = BasicClient::new(config.client_id.clone())
            .set_client_secret(config.client_secret.clone())
            .set_auth_uri(config.auth_uri.clone())
            .set_token_uri(config.token_uri.clone())
            .set_redirect_uri(config.redirect_uri.clone());
    
        let (auth_url, csrf_token) = client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("profile".to_string()))
            .add_scope(Scope::new("email".to_string()))
            .set_pkce_challenge(pkce_challenge)
            .url();

        let csrf_cookie = Cookie::build(("oauth_csrf", csrf_token.secret().clone()))
            .path("/")
            .http_only(true)
            .same_site(SameSite::Lax)
            .secure(true)
            .max_age(time::Duration::minutes(5));
        
        let pkce_cookie =  Cookie::build(("oauth_pkce_verifier", pkce_verifier.secret().clone()))
            .path("/")
            .http_only(true)
            .same_site(SameSite::Lax)
            .secure(true)
            .max_age(time::Duration::minutes(5));

        let jar = cookie_jar.add(csrf_cookie).add(pkce_cookie);

        Ok((jar, Redirect::to(auth_url.as_str())))

    } else {
        Err(AuthError::InternalServerError(Some("Google OAuth is not configured, unable to continue with the authorization flow.".to_string())))
    }

}   

// #######################################################################################################################################################
// Google OAuth Flow Callback Handler
// #######################################################################################################################################################

#[derive(Debug, Deserialize)]
pub struct GoogleOAuthCallbackParams {
    code: String,
    state: String,
}

#[derive(Deserialize)]
pub struct GoogleAccessTokenPayload {
    pub email_verified: Option<bool>,
    pub email: Option<String>,
    pub family_name: Option<String>,
    pub given_name: Option<String>,
    pub name: Option<String>,
    pub picture: Option<String>,
    pub sub: String,

}

pub async fn google_oauth_callback_handler(
    data: Arc<AppState>,
    cookie_jar: CookieJar,
    callback_params: GoogleOAuthCallbackParams,
) -> Result<impl IntoResponse, AuthError> {
    let csrf_cookie = cookie_jar
        .get("oauth_csrf")
        .ok_or(AuthError::CSRFTokenMismatch)?;

    let pkce_cookie = cookie_jar
        .get("oauth_pkce_verifier")
        .ok_or(AuthError::OAuthError(Some("Unable to get PKCE cookie.".to_string())))?;

    if csrf_cookie.value() != callback_params.state {
        return Err(AuthError::CSRFTokenMismatch);
    }

    let config = data
        .google_oauth_config
        .as_ref()
        .ok_or_else(|| {
            eprintln!("Google OAuth config is missing.");
            AuthError::OAuthError(Some("OAuth config not found.".to_string()))
        })?;

    let oauth_client = BasicClient::new(config.client_id.clone())
        .set_client_secret(config.client_secret.clone())
        .set_auth_uri(config.auth_uri.clone())
        .set_token_uri(config.token_uri.clone())
        .set_redirect_uri(config.redirect_uri.clone());

    // The following token exchange returns this on success.
    // Ok(
    //     StandardTokenResponse {
    //         access_token: AccessToken([redacted]),
    //         token_type: Bearer,
    //         expires_in: Some(
    //             3599,
    //         ),
    //         refresh_token: None,
    //         scopes: Some(
    //             [
    //                 Scope(
    //                     "https://www.googleapis.com/auth/userinfo.profile",
    //                 ),
    //                 Scope(
    //                     "https://www.googleapis.com/auth/userinfo.email",
    //                 ),
    //                 Scope(
    //                     "openid",
    //                 ),
    //             ],
    //         ),
    //         extra_fields: EmptyExtraTokenFields,
    //     },
    // )
    let token = oauth_client
        .exchange_code(AuthorizationCode::new(callback_params.code))
        .set_pkce_verifier(PkceCodeVerifier::new(pkce_cookie.value().to_string()))
        .request_async(&data.http_client)
        .await
        .map_err(|e| AuthError::OAuthError(Some(e.to_string())))?;

    // The following request returns this upon success
    // {
    //     "email": String("corrado@mazzarelli.biz"),
    //     "email_verified": Bool(true),
    //     "family_name": String("Mazzarelli"),
    //     "given_name": String("Corrado"),
    //     "name": String("Corrado “Cory” Mazzarelli"),
    //     "picture": String("https://lh3.googleusercontent.com/a/ACg8ocJ-vpjiao2CsPCixOZKm7Oc1U2SecYPxdmhW1hNCL0WaQayvA=s96-c"),
    //     "sub": String("123520143226431893380"),
    // }
    let user_info = data
        .http_client
        .get("https://www.googleapis.com/oauth2/v3/userinfo")
        .bearer_auth(token.access_token().secret())
        .send()
        .await
        .map_err(|e| AuthError::OAuthError(Some(e.to_string())))?
        .json::<GoogleAccessTokenPayload>()
        .await
        .map_err(|e| AuthError::OAuthError(Some(e.to_string())))?;

    let email = user_info.email.as_ref().ok_or_else(|| {
        AuthError::OAuthError(Some("Google did not return an email address when signing in. The app cannot handle this. Try signing in without Google or try again later.".to_string()))
    })?;

    let user = get_user(email, &data.db).await?;

    match user {
        Some(user) => {
            let jar = login_user(user, &data, cookie_jar).await?;
            Ok((jar, Redirect::to("/dashboard")).into_response())
        }
        None => Err(AuthError::AccountNotFound),
    }
}
// #######################################################################################################################################################
// Logout
// #######################################################################################################################################################

pub async fn logout_handler(
    cookie_jar: CookieJar,
    authorized_user: AuthorizedUser,
    data: Arc<AppState>,
) -> Result<impl IntoResponse, AuthError> {

    let refresh_token = cookie_jar
        .get("refresh_token")
        .map(|cookie| cookie.value().to_string())
        .ok_or_else(|| AuthError::NotLoggedIn)?;

    let refresh_token_details = verify_jwt_token(data.env.refresh_token_public_key.to_owned(), &refresh_token)
            .map_err(|e| AuthError::InternalServerError(Some(format!("{:?}", e))))?;

    let mut redis_client = data
        .redis_client
        .get_multiplexed_async_connection()
        .await
        .map_err(|e: RedisError| {
            AuthError::InternalServerError(Some(format!("Redis error: {}", e)))
        })?;

    redis_client
        .del::<_, ()>(&[
            refresh_token_details.token_uuid.to_string(),
            authorized_user.access_token_uuid.to_string(),
        ])
        .await
        .map_err(|e: RedisError| {
            AuthError::InternalServerError(Some(format!("Redis error: {}", e)))
        })?;

    let access_cookie = Cookie::build(("access_token", ""))
        .path("/")
        .max_age(time::Duration::minutes(-1))
        .same_site(SameSite::Lax)
        .http_only(true);
    let refresh_cookie = Cookie::build(("refresh_token", ""))
        .path("/")
        .max_age(time::Duration::minutes(-1))
        .same_site(SameSite::Lax)
        .http_only(true);

    let logged_in_cookie = Cookie::build(("logged_in", "true"))
        .path("/")
        .max_age(time::Duration::minutes(-1))
        .same_site(SameSite::Lax)
        .http_only(false);

    let mut headers = HeaderMap::new();
    headers.append(
        header::SET_COOKIE,
        access_cookie.to_string().parse().unwrap(),
    );
    headers.append(
        header::SET_COOKIE,
        refresh_cookie.to_string().parse().unwrap(),
    );
    headers.append(
        header::SET_COOKIE,
        logged_in_cookie.to_string().parse().unwrap(),
    );

    let mut response = Redirect::to("/").into_response();
    response.headers_mut().extend(headers);
    Ok(response)
}

// #######################################################################################################################################################
// Utility Functions
// #######################################################################################################################################################

async fn save_token_data_to_redis(
    data: &Arc<AppState>,
    token_details: &TokenDetails,
    max_age: i64,
) -> Result<(), RedisError> {
    let mut redis_client = data
        .redis_client
        .get_multiplexed_async_connection()
        .await?;
    redis_client
        .set_ex::<_, _, ()>(
            token_details.token_uuid.to_string(),
            token_details.user_id.to_string(),
            (max_age * 60) as u64,
        )
        .await?;
    Ok(())
}

/// Gets a user from the database
pub async fn get_user(
    email: &str,
    db: &Pool<Postgres>,
) -> Result<Option<User>, AuthError> {
    sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE email = $1",
        email.to_ascii_lowercase()
    )
        .fetch_optional(db)
        .await
        .map_err(|e| AuthError::InternalServerError(Some(format!("Database error: {}", e))))
}

async fn login_user(
    user: User,
    data: &Arc<AppState>,
    cookie_jar: CookieJar,
) -> Result<CookieJar, AuthError> {
    let access_token_details = generate_jwt_token(
        user.id,
        data.env.access_token_max_age,
        data.env.access_token_private_key.to_owned()
    ).map_err(|e: jsonwebtoken::errors::Error| {
            AuthError::InternalServerError(Some(format!("JWT error: {}", e)))
    })?;

    let refresh_token_details = generate_jwt_token(
        user.id,
        data.env.refresh_token_max_age,
        data.env.refresh_token_private_key.to_owned(),
    ).map_err(|e: jsonwebtoken::errors::Error| {
        AuthError::InternalServerError(Some(format!("JWT error: {}", e)))
    })?;

    save_token_data_to_redis(&data, &access_token_details, data.env.access_token_max_age).await
        .map_err(|e: RedisError| {
        AuthError::InternalServerError(Some(format!("Redis error: {}", e)))
        })?;

    save_token_data_to_redis(
        &data,
        &refresh_token_details,
        data.env.refresh_token_max_age,
    )
        .await    
        .map_err(|e: RedisError| {
            AuthError::InternalServerError(Some(format!("Redis error: {}", e)))
        })?;

    let access_cookie = Cookie::build(
        ("access_token",
        access_token_details.token.clone().unwrap_or_default()),
    )
        .path("/")
        .max_age(time::Duration::minutes(data.env.access_token_max_age * 60))
        .same_site(SameSite::Lax)
        .http_only(true);

    let refresh_cookie = Cookie::build(
        ("refresh_token",
        refresh_token_details.token.unwrap_or_default()),
    )
        .path("/")
        .max_age(time::Duration::minutes(data.env.refresh_token_max_age * 60))
        .same_site(SameSite::Lax)
        .http_only(true);

    let logged_in_cookie = Cookie::build(("logged_in", "true"))
        .path("/")
        .max_age(time::Duration::minutes(data.env.access_token_max_age * 60))
        .same_site(SameSite::Lax)
        .http_only(false);

    Ok(cookie_jar
        .add(access_cookie)
        .add(refresh_cookie)
        .add(logged_in_cookie))
}