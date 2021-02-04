use discord::{api::Api, model::gateway::Event, Error};
use futures::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let token = std::env::var("TOKEN").expect("missing token");

    let api = Api::new(&token);
    let (mut shard, conn) = discord::gateway::with_rest_client(&token, api.clone());

    let conn = tokio::spawn(conn);

    while let Some(event) = shard.next().await {
        match event {
            Event::MessageCreate(msg) => {
                if msg.content.starts_with("ping") {
                    let msg = api.wrap(*msg);
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
