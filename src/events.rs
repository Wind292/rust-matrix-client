use core::error;

use serde_json::{Value, json, value};

use crate::{errors, utils::*};
use crate::auth::AuthState;

pub struct EventState { 
    account_data: Option<Value>,
    device_one_time_keys_count: Option<Value>,
    device_unused_fallback_key_types: Option<Vec<Value>>,
    next_batch: Option<String>,
    presence: Option<Value>,
}

impl EventState {
    pub fn new() -> Self { 
        EventState { account_data: None, device_one_time_keys_count: None, device_unused_fallback_key_types: None, next_batch: None, presence: None }
    }

    pub async fn sync(&mut self, mut client_state: AuthState) -> Result<(), Box<dyn std::error::Error>>  {
        let resp = client_state.auth_get("v3/sync").await?;

        let account_data = resp.get("account_data");
        let device_one_time_keys_count = resp.get("device_one_time_keys_count");
        let device_unused_fallback_key_types = resp.get("device_unused_fallback_key_types");
        let next_batch = resp.get("next_batch");
        let presence = resp.get("presence");

        println!("{:?}", account_data);


        self.account_data = account_data.cloned();
        self.device_one_time_keys_count = device_one_time_keys_count.cloned();
        self.device_unused_fallback_key_types = device_unused_fallback_key_types.and_then(|t| t.as_array()).cloned();
        self.next_batch = next_batch.cloned().and_then(|t| Some(t.to_string()));
        self.presence = presence.cloned();
        
        Ok(())
    }

    // this is bare bones see
    // 8.2 Creation for more additions 
    pub async fn create_room(&self, mut auth_state: AuthState, name: &str, invites: Vec<&str>, preset: &str) -> Result<(), Box<dyn std::error::Error>> {
        let body = json!({
            // "creation_content": {
            //     "m.federate": false
            // },
            "preset": preset,
            "name": name,
            "invite": invites,
        });

        let response = auth_state.auth_post("v3/createRoom", &body.to_string()).await?;
        
        if response.get("errcode").is_some() {
            return Err(errors::MatrixError::json(response).into())
        }
        

        Ok(())
    }

}

