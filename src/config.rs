fn get_env_var(var_name: &str) -> String {
    std::env::var(var_name).unwrap_or_else(|_| panic!("{} must be set as an environment variable.", var_name))
}

#[derive(Debug, Clone)]
pub struct SecretsConfig {
    pub is_demo_mode: bool,
    pub signup_licensing_key: String,
    pub queue_signup_key: String,
    pub server_port: i64,
    pub database_url: String,
    pub redis_url: String,

    pub smtp_server_host: String,
    pub smtp_user_login: String,
    pub smtp_user_password: String,
    pub smtp_user_email: String,

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

        let smtp_server_host = get_env_var("SMTP_SERVER_HOST");
        let smtp_user_login = get_env_var("SMTP_USER_LOGIN");
        let smtp_user_password = get_env_var("SMTP_USER_PASSWORD");
        let smtp_user_email = get_env_var("SMTP_USER_EMAIL");

        let access_token_private_key = get_env_var("ACCESS_TOKEN_PRIVATE_KEY");
        let access_token_public_key = get_env_var("ACCESS_TOKEN_PUBLIC_KEY");
        let access_token_expires_in = get_env_var("ACCESS_TOKEN_EXPIRED_IN");
        let access_token_max_age = get_env_var("ACCESS_TOKEN_MAXAGE");

        let refresh_token_private_key = get_env_var("REFRESH_TOKEN_PRIVATE_KEY");
        let refresh_token_public_key = get_env_var("REFRESH_TOKEN_PUBLIC_KEY");
        let refresh_token_expires_in = get_env_var("REFRESH_TOKEN_EXPIRED_IN");
        let refresh_token_max_age = get_env_var("REFRESH_TOKEN_MAXAGE");

        SecretsConfig {
            is_demo_mode,
            signup_licensing_key,
            queue_signup_key,
            server_port,
            database_url,
            redis_url,
            smtp_server_host,
            smtp_user_login,
            smtp_user_password,
            smtp_user_email,
            access_token_private_key,
            access_token_public_key,
            refresh_token_private_key,
            refresh_token_public_key,
            access_token_expires_in,
            refresh_token_expires_in,
            access_token_max_age: access_token_max_age.parse::<i64>().unwrap(),
            refresh_token_max_age: refresh_token_max_age.parse::<i64>().unwrap(),
        }
    }
}
