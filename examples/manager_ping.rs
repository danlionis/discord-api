use std::sync::Arc;

use discord::{
    model::gateway::Event,
    rest::{client::Client, CreateMessageParams},
    Error,
};

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
    match event {
        Event::MessageCreate(msg) => {
            if msg.content.starts_with("!ping") {
                let _ = rest
                    .create_message(
                        msg.channel_id,
                        CreateMessageParams::default()
                            .content("Pong")
                            .reference(msg.reference()),
                    )
                    .await;
            }
            if msg.content.starts_with("!react") {
                let _ = rest
                    .create_reaction(msg.channel_id, msg.id, "%F0%9F%98%80".to_owned())
                    .await;
            }
        }
        Event::ThreadCreate(channel) => {
            dbg!(channel);
        }
        _ => {}
    }
}
