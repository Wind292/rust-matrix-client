use std::collections::HashMap;

use serde_json::Value;
mod errors;

pub struct Server { 
    pub adress: String,
}

impl Server {
    pub async fn GET(&self, path: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let resp = reqwest::get(self.adress.clone() + "/_matrix/client/v3/" + path)
            .await?
            .json::<serde_json::Value>()
            .await?;
        println!("{}",self.adress.clone() + "/_matrix/client/v3/" + path);
        Ok(resp)
    }
    pub async fn POST(&self, path: &str, data: &[u8]) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let data_clone = data.to_vec();
        let resp = client.post(self.adress.clone() + "/_matrix/client/v3/" + path)
            .body(data_clone)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        println!("{}", resp);
        Ok(resp)
    }

    pub async fn login_password(&self, username: &str, password: &str) -> Result<(), Box<dyn std::error::Error>> {
        let resp = self.GET("login").await?;
        let supported_auths = resp.as_array();
        
        let mut supports_password_login = false;
        match supported_auths {
            Some(arr) => {
                for auth in arr { 
                    if auth.as_str().unwrap_or("Not A String") == "m.login.password" { 
                        supports_password_login = true;
                    } 
                }
            }
            None => return Err(errors::AuthError::InvalidJson.into())
        }

        if supports_password_login == false { 
            return Err(errors::AuthError::UnsupportedAuthType.into());
        }
        // Now we can be sure that the server supports password logins 

        Ok(())
    } 
}

