use discord::{model::gateway::Event, rest::Rest, Error};
use futures::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let token = std::env::var("TOKEN").expect("missing token");

    let rest = Rest::new(&token);
    let (mut shard, conn) = discord::gateway::with_rest_client(&token, rest.clone());

    let _conn = tokio::spawn(conn);

    let mut bot_ping = 0;

    while let Some(event) = shard.next().await {
        match event {
            Event::Raw(raw) => {
                dbg!(raw);
            }
            Event::Ping(p) => {
                bot_ping = p;
                dbg!(p);
            }
            Event::MessageCreate(msg) => {
                if msg.content.starts_with("ping") {
                    let msg = rest.wrap(*msg);
                    msg.reply(format!("Pong: {}ms", bot_ping.to_string(),))
                        .await?;
                }
            }
            _ => {}
        }
    }

    Ok(())
}
