use std::{error::Error, sync::Arc};

use discord::{model::gateway::Event, rest::CreateMessageParams};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let token = std::env::var("TOKEN").expect("missing token");

    env_logger::init();

    let mut manager = discord::manager::connect(token).await?;
    log::info!("manager created");

    while let Ok(event) = manager.recv().await {
        let rest = Arc::clone(manager.rest());

        tokio::spawn(async move {
            if let Event::MessageCreate(msg) = event {
                if msg.content.contains("ping") {
                    let _ = rest
                        .create_message(
                            msg.channel_id,
                            CreateMessageParams::default()
                                .content("Pong")
                                .reference(msg.reference()),
                        )
                        .await;
                }
            }
        });
    }

    Ok(())
}
