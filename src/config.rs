use lettre::transport::smtp::{client, commands::Auth};
use oauth2::{basic::BasicClient, AuthUrl, Client, ClientId, ClientSecret, EndpointNotSet, EndpointSet, RedirectUrl, TokenUrl};

pub fn get_env_var(var_name: &str) -> String {
    std::env::var(var_name).unwrap_or_else(|_| panic!("{} must be set as an environment variable. Use an empty string if this is for optional functionality.", var_name))
}

#[derive(Debug, Clone)]
pub struct GoogleOAuthConfig {
    pub client_id: ClientId,
    pub client_secret: ClientSecret,
    pub auth_uri: AuthUrl,
    pub token_uri: TokenUrl,
    pub redirect_uri: RedirectUrl,
}
impl GoogleOAuthConfig {
    pub fn init() -> Option<GoogleOAuthConfig> {
        let client_id = get_env_var("GOOGLE_OAUTH_CLIENT_ID");
        let client_secret = get_env_var("GOOGLE_OAUTH_CLIENT_SECRET");
        let auth_uri = get_env_var("GOOGLE_OAUTH_AUTH_URI");
        let token_uri = get_env_var("GOOGLE_OAUTH_TOKEN_URI");
        let redirect_uri = get_env_var("GOOGLE_OAUTH_REDIRECT_URI");

        match (
            client_id.as_str(),
            client_secret.as_str(),
            auth_uri.as_str(),
            token_uri.as_str(),
            redirect_uri.as_str(),
        ) {
            ("", "", "", "", "") => {
                println!("\nGoogle OAuth functionality disabled since all environment variables were left blank.");
                None
            }
            ("", _, _, _, _) => {
                println!("\nGoogle OAuth functionality disabled: missing GOOGLE_OAUTH_CLIENT_ID.");
                None
            }
            (_, "", _, _, _) => {
                println!("\nGoogle OAuth functionality disabled: missing GOOGLE_OAUTH_CLIENT_SECRET.");
                None
            }
            (_, _, "", _, _) => {
                println!("\nGoogle OAuth functionality disabled: missing GOOGLE_OAUTH_AUTH_URI.");
                None
            }
            (_, _, _, "", _) => {
                println!("\nGoogle OAuth functionality disabled: missing GOOGLE_OAUTH_TOKEN_URI.");
                None
            }
            (_, _, _, _, "") => {
                println!("\nGoogle OAuth functionality disabled: missing GOOGLE_OAUTH_REDIRECT_URI.");
                None
            }
            (client_id, client_secret, auth_uri, token_uri, redirect_uri) => {
                println!(
                    "\nGoogle OAuth functionality is enabled.",
                );

                Some(GoogleOAuthConfig {
                    client_id: ClientId::new(client_id.to_string()),
                    client_secret: ClientSecret::new(client_secret.to_string()),
                    auth_uri: AuthUrl::new(auth_uri.to_string()).expect("Unable to parse GOOGLE_OAUTH_AUTH_URI."),
                    token_uri: TokenUrl::new(token_uri.to_string()).expect("Unable to parse GOOGLE_OAUTH_TOKEN_URI."),
                    redirect_uri: RedirectUrl::new(redirect_uri.to_string()).expect("Unable to parse GOOGLE_OAUTH_REDIRECT_URI."),
                })
            }
        }
    }
}


#[derive(Debug, Clone)]
pub struct SecretsConfig {
    pub is_demo_mode: bool,
    pub signup_licensing_key: String,
    pub queue_signup_key: String,
    pub server_port: i64,
    pub database_url: String,
    pub redis_url: String,

    pub access_token_private_key: String,
    pub access_token_public_key: String,
    pub access_token_expires_in: String,
    pub access_token_max_age: i64,

    pub refresh_token_private_key: String,
    pub refresh_token_public_key: String,
    pub refresh_token_expires_in: String,
    pub refresh_token_max_age: i64,
}

impl SecretsConfig {
    pub fn init() -> SecretsConfig {
        let is_demo_mode = get_env_var("DEMO_MODE_ACTIVE").to_lowercase().trim().parse().expect("DEMO_MODE_ACTIVE should be TRUE or FALSE.");
        let signup_licensing_key = get_env_var("SIGNUP_LICENSING_KEY");
        let queue_signup_key = get_env_var("QUEUE_SIGNUP_KEY");
        let server_port = get_env_var("SERVER_PORT").parse::<i64>().expect("Server port (ENV_VAR=SERVER_PORT) should be an integer.");
        let database_url = get_env_var("DATABASE_URL");
        let redis_url = get_env_var("REDIS_URL");

        let access_token_private_key = get_env_var("ACCESS_TOKEN_PRIVATE_KEY");
        let access_token_public_key = get_env_var("ACCESS_TOKEN_PUBLIC_KEY");
        let access_token_expires_in = get_env_var("ACCESS_TOKEN_EXPIRED_IN");
        let access_token_max_age = get_env_var("ACCESS_TOKEN_MAXAGE").parse::<i64>().expect("Access token max age (ENV_VAR=ACCESS_TOKEN_MAXAGE) should be an integer.");

        let refresh_token_private_key = get_env_var("REFRESH_TOKEN_PRIVATE_KEY");
        let refresh_token_public_key = get_env_var("REFRESH_TOKEN_PUBLIC_KEY");
        let refresh_token_expires_in = get_env_var("REFRESH_TOKEN_EXPIRED_IN");
        let refresh_token_max_age = get_env_var("REFRESH_TOKEN_MAXAGE").parse::<i64>().expect("Refresh token max age (ENV_VAR=REFRESH_TOKEN_MAXAGE) should be an integer.");

        SecretsConfig {
            is_demo_mode,
            signup_licensing_key,
            queue_signup_key,
            server_port,
            database_url,
            redis_url,
            access_token_private_key,
            access_token_public_key,
            refresh_token_private_key,
            refresh_token_public_key,
            access_token_expires_in,
            refresh_token_expires_in,
            access_token_max_age,
            refresh_token_max_age,
        }
    }
}
