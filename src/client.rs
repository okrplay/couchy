use reqwest::StatusCode;
use serde::Serialize;
use std::time::Duration;
use tokio::time;

#[derive(Clone)]
pub struct Client {
    url: String,
    reqwest_client: reqwest::Client,
}

#[derive(Serialize)]
struct CouchCredentials {
    username: String,
    password: String,
}

pub enum CouchAuthError {
    Unauthorized,
    InternalError(String),
}

impl Client {
    pub async fn new<S: ToString>(url: S) -> Client {
        Client {
            url: url.to_string(),
            reqwest_client: reqwest::Client::builder()
                .cookie_store(true)
                .build()
                .unwrap(),
        }
    }

    pub async fn with_auth<S: ToString>(
        self,
        username: S,
        password: S,
    ) -> Result<(), CouchAuthError> {
        let credentials = CouchCredentials {
            username: username.to_string(),
            password: password.to_string(),
        };

        let first_auth = self
            .reqwest_client
            .post(&format!("{}/_session", self.url))
            .json(&credentials)
            .send();

        match first_auth.await.unwrap().status() {
            StatusCode::OK => {
                // TODO: Proper logging
                tokio::spawn(async move {
                    let mut interval = time::interval(Duration::from_secs(570));
                    loop {
                        interval.tick().await;
                        let req = self.clone()
                            .reqwest_client
                            .post(&format!("{}/_session", self.url))
                            .json(&credentials)
                            .send();
                        println!("requested new couchdb token status {}", req.await.unwrap().status());
                    }
                });
                Ok(())
            }

            StatusCode::UNAUTHORIZED => Err(CouchAuthError::Unauthorized),

            _ => {
                Err(CouchAuthError::InternalError(
                    "couchy crashed during first auth (unrecognized status code)".to_string(),
                ))
            }
        }
    }
}
