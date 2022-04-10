use std::error::Error;

use discord::rest::client::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let token = std::env::var("TOKEN").expect("missing token");

    env_logger::init();

    let client = Client::new(token);
    let user = client.get_current_user().await?;
    println!("tag= {} id= {}", user.tag(), user.id);

    Ok(())
}
