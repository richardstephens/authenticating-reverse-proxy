use std::collections::{HashMap, HashSet};

use lazy_static::lazy_static;
use serde::Deserialize;
use thiserror::Error;
use tokio::sync::RwLock;

lazy_static! {
    static ref USER_MAP:RwLock<HashMap<String,String>> = RwLock::new(HashMap::new());
    static ref ORG_MEMBERS:RwLock<HashSet<String>> = RwLock::new(HashSet::new());
}

#[derive(Deserialize, Debug)]
pub struct GhUserResponse {
    pub login: String,
}

#[derive(Deserialize, Debug)]
pub struct GhOrgMember {
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

async fn get_org_members_page(org: &String, token: &String, page: u32) -> Vec<GhOrgMember> {
    let mut auth_header_value: String = "Bearer ".to_string();
    auth_header_value.push_str(token);
    let client = reqwest::Client::new();
    let mut req_url = "https://api.github.com/orgs/".to_string();
    req_url.push_str(&org);
    req_url.push_str("/members?per_page=100&page=");
    req_url.push_str(&page.to_string());
    println!("URL: {}", req_url);
    let resp = client.get(&req_url)
        .header("User-Agent", "Authenticating Reverse Proxy (https://github.com/richardstephens/authenticating-reverse-proxy)")
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("Authorization", &auth_header_value)
        .send().await;

    let resp_content = resp.unwrap().json::<Vec<GhOrgMember>>().await.unwrap();
    return resp_content;
}

pub async fn get_org_members(org: String, token: String) {
    let mut page: u32 = 0;
    loop {
        let page_contents = get_org_members_page(&org, &token, page).await;
        println!("Reading members for org: {}. Page {} got {} entries", org, page, page_contents.len());
        let mut writeable_org_members = ORG_MEMBERS.write().await;
        for org_member in &page_contents {
            writeable_org_members.insert(org_member.login.clone());
        }
        if page_contents.len() == 0 {
            break;
        } else {
            page = page + 1;
        }
    }
    println!("Got {} org members", ORG_MEMBERS.read().await.len());
}

pub async fn check_token(user: String, token: String) -> bool {
    if !ORG_MEMBERS.read().await.contains(&user) {
        println!("user not in org");
        return false;
    }
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
                        println!("Github token did not match user: {}", token_username);
                        false
                    }
                }
                Err(_) => false
            }
        }
    }
}
