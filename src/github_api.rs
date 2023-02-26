use std::collections::HashMap;

use lazy_static::lazy_static;
use serde::Deserialize;
use thiserror::Error;
use tokio::sync::RwLock;

lazy_static! {
    static ref USER_MAP:RwLock<HashMap<String,String>> = RwLock::new(HashMap::new());
}

#[derive(Deserialize, Debug)]
pub struct GhUserResponse {
    pub login: String,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Github API error")]
    GhApiError(String),
    #[error("Github Authentication failed")]
    GhAuthFailed,
}

impl std::convert::From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::GhApiError(err.to_string())
    }
}


async fn get_user_for_token(token: &String) -> Result<String, Error> {
    let mut auth_header_value: String = "Bearer ".to_string();
    auth_header_value.push_str(token);

    let client = reqwest::Client::new();
    let resp = client.get("https://api.github.com/user")
        .header("User-Agent", "Authenticating Reverse Proxy (https://github.com/richardstephens/authenticating-reverse-proxy)")
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("Authorization", auth_header_value)
        .send().await?;

    return if resp.status().is_success() {
        let resp_content = resp.json::<GhUserResponse>().await?;
        Ok(resp_content.login)
    } else if resp.status().as_u16() == 403 {
        Err(Error::GhAuthFailed)
    } else {
        Err(Error::GhApiError(format!("GH status code: {}", resp.status().as_u16())))
    };
}

pub async fn check_token(user: String, token: String) -> bool {
    let user_details = {
        USER_MAP.read().await.get(&user).map(|x| x.clone())
    };
    match user_details {
        Some(pass) => {
            if pass == token {
                println!("User: {} authenticated from cache", user);
                true
            } else {
                false
            }
        }
        None => {
            let gh_response = get_user_for_token(&token).await;
            match gh_response {
                Ok(token_username) => {
                    if token_username == user {
                        println!("User: {} authenticated successfully", token_username);
                        let mut writable_user_map = USER_MAP.write().await;
                        writable_user_map.insert(token_username, token.clone());
                        true
                    } else {
                        false
                    }
                }
                Err(_) => false
            }
        }
    }
}
