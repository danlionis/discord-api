use std::sync::Arc;

use discord::Error;
use twilight_http::{request::channel::reaction::RequestReactionType, Client};
use twilight_model::gateway::event::Event;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let token = std::env::var("TOKEN").expect("missing token");

    env_logger::init();

    let mut manager = discord::manager::connect(token).await?;

    while let Ok(event) = manager.recv().await {
        let rest = Arc::clone(manager.rest());
        tokio::spawn(handle_event(rest, event));
    }

    Ok(())
}

async fn handle_event(rest: Arc<Client>, event: Event) {
    if let Event::MessageCreate(msg) = event {
        if msg.content.starts_with("!ping") {
            let _ = rest
                .create_message(msg.channel_id)
                .content("pong")
                .unwrap()
                .exec()
                .await;
        }
        if msg.content.starts_with("!react") {
            let _ = rest
                .create_reaction(
                    msg.channel_id,
                    msg.id,
                    &RequestReactionType::Unicode { name: "😀" },
                )
                .exec()
                .await;
        }
    }
}
