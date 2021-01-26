use discord::{model::gateway::Event, rest::RestClient, Error};
use futures::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let token = std::env::var("TOKEN").expect("missing token");

    let rest_client = RestClient::new(&token);
    let (mut shard, conn) = discord::gateway::with_rest_client(&token, rest_client.clone());

    let conn = tokio::spawn(conn);

    while let Some(event) = shard.next().await {
        match event {
            Event::MessageCreate(msg) => {
                if msg.content.starts_with("ping") {
                    let msg = rest_client.wrap(*msg);
                    msg.reply(format!(
                        "Pong: {}ms",
                        shard
                            .ping()
                            .expect("we received a message so we must be connected")
                            .to_string(),
                    ))
                    .await?;
                }
            }
            _ => {}
        }
    }

    Ok(())
}
