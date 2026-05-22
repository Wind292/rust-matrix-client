pub async fn unauth_get(server_address: &str, path: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let resp = reqwest::get(server_address.to_string() + "/_matrix/client/" + path)
        .await?
        .json::<serde_json::Value>()
        .await?;
    Ok(resp)
}

pub async fn unauth_post(
    server_address: &str,
    path: &str,
    data: &str,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let data_clone = data.to_owned();
    let resp = client
        .post(server_address.to_string() + "/_matrix/client/" + path)
        .body(data_clone)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;
    Ok(resp)
}
