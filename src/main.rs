mod auth;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = auth::Server { adress: "http://localhost:8008".to_string() };

    // let x = server.POST("login", ).await?;
    let x = server.login_password("username", "password").await?;
    println!("{:?}", x);
    Ok(())
}  