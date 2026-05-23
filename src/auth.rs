use reqwest::header::AUTHORIZATION;
use serde_json::Value;
use serde_json::json;

use crate::errors;
use crate::errors::CustomError;
use crate::utils::unauth_get;
use crate::utils::unauth_post;


pub struct AuthState {
    pub server_address: String,
    pub user_id: String,
    pub token: String,
    pub refresh_token: Option<String>,
    pub device_id: Option<String>,
    pub expiration: Option<i64>
}

impl AuthState {
    pub async fn login_password (
        server_address: &str,
        username: &str,
        password: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let resp = unauth_get(server_address, "v3/login").await?;
        let supported_auths = resp.get("flows").unwrap_or_default().as_array();

        let mut supports_password_login = false;
        match supported_auths {
            Some(arr) => {
                for auth in arr {
                    let auth_string = auth //
                        .get("type")
                        .unwrap_or_default()
                        .as_str()
                        .unwrap_or("Not A String");

                    if auth_string == "m.login.password" {
                        supports_password_login = true;
                    }
                }
            }
            None => return Err(CustomError::InvalidJson.into()),
        }

        if supports_password_login == false {
            return Err(CustomError::UnsupportedAuthType.into());
        }
        // Now we can be sure that the server supports password logins

        let login_packet_json = json!(
            {
                "identifier": {
                    "type": "m.id.user",
                    "user": username,
                },
                "initial_device_display_name": "rust-matrix-client",
                "password": password,
                "type": "m.login.password"
            }
        );
            
        let post_response = unauth_post(server_address, "v3/login", &login_packet_json.to_string()).await?;

        if post_response.get("errcode").is_some() { // Server returned an errorcode, return it as MatrixError
            return Err(errors::MatrixError::json(post_response).into())
        }


        let token = post_response.get("access_token").and_then(|t| t.as_str()).and_then(|t| Some(t.to_string()));
        let user_id = post_response.get("user_id").and_then(|t| t.as_str()).and_then(|t| Some(t.to_string()));
        let refresh_token = post_response.get("refresh_token").and_then(|t| t.as_str()).and_then(|t| Some(t.to_string()));
        let device_id = post_response.get("device_id").and_then(|t| t.as_str()).and_then(|t| Some(t.to_string()));
        let expiration = post_response.get("expires_in_ms").and_then(|t| t.as_i64());


        let unwrapped_token = match token {
            Some(t) => t,
            None => return Err(CustomError::MissingTokenInResponse.into()),
        };

        let unwrapped_user_id = match user_id {
            Some(t) => t,
            None => return Err(CustomError::MissingUserIdInResponse.into()),
        };

        let client = AuthState {
            server_address: server_address.to_string(),
            refresh_token: refresh_token,
            device_id: device_id,
            expiration: expiration,
            user_id: unwrapped_user_id,
            token: unwrapped_token,
        };

        Ok(client)
    }

    pub async fn auth_get(&mut self, path: &str) -> Result<Value, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();

        let resp = client.get(self.server_address.to_string() + "/_matrix/client/" + path)
            .header(AUTHORIZATION, format!("Bearer {}", self.get_token().await))
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;
        
        if resp.get("errcode").is_some() {
            return Err(errors::MatrixError::json(resp).into())
        }

        Ok(resp)
    }

    pub async fn auth_post(&mut self, path: &str, body: &str) -> Result<Value, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();

        let resp = client.post(self.server_address.to_string() + "/_matrix/client/" + path)
            .header(AUTHORIZATION, format!("Bearer {}", self.get_token().await))
            .body(body.to_owned())
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        if resp.get("errcode").is_some() {
            return Err(errors::MatrixError::json(resp).into())
        }    
        
        Ok(resp)
    }

    pub async fn auth_put(&mut self, path: &str, body: &str) -> Result<Value, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();

        let resp = client.put(self.server_address.to_string() + "/_matrix/client/" + path)
            .header(AUTHORIZATION, format!("Bearer {}", self.get_token().await))
            .body(body.to_owned())
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        if resp.get("errcode").is_some() {
            return Err(errors::MatrixError::json(resp).into())
        }

        Ok(resp)
    }

    async fn get_token(&mut self) -> String {
        self.token.clone()
    }

    

}



pub async fn get_oauth2_support(server_address: &str) -> Result<bool, Box<dyn std::error::Error>> { 
    let response = reqwest::get(server_address.to_string() + "/_matrix/client/v1/auth_metadata" ).await?;
    let response_code = response.status().as_u16();

    match response_code {
        404 => return Ok(false),
        200 => return Ok(true),
        429 => return Err(CustomError::RateLimited.into()),
        _ => return Err(CustomError::AuthMetadataQueryUnrecognizedCode.into())
    }
}

