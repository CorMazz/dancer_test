use std::sync::Arc;

use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::{
    http::{header, HeaderMap},
    response::{IntoResponse, Redirect},
};
use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    CookieJar,
};
use rand_core::OsRng;

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

    let user_exists: Option<bool> =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)")
            .bind(email.to_ascii_lowercase())
            .fetch_one(&data.db)
            .await
            .map_err(|e| AuthError::InternalServerError(Some(format!("Database error: {}", e))))?;

    if let Some(exists) = user_exists {
        if exists {
            return Err(AuthError::DuplicateEmail);
        }
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
    email: String,
    password: String,
) -> Result<impl IntoResponse, AuthError> {

    let user = sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE email = $1",
        email.to_ascii_lowercase()
    )
    .fetch_optional(&data.db)
    .await
    .map_err(|e| AuthError::InternalServerError(Some(format!("Database error: {}", e))))?
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

    let mut response = Redirect::to("/dashboard").into_response();
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

    response.headers_mut().extend(headers);
    Ok(response)
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
        .del(&[
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
        .set_ex(
            token_details.token_uuid.to_string(),
            token_details.user_id.to_string(),
            (max_age * 60) as u64,
        )
        .await?;
    Ok(())
}