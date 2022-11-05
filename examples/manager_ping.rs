use discord::{
    manager::{self, Manager},
    model::gateway::{event::DispatchEvent, Intents},
    proto::Config,
    Error,
};
use std::convert::TryFrom;
use twilight_http::request::channel::reaction::RequestReactionType;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let token = std::env::args().skip(1).next().expect("missing token");

    env_logger::init();

    let config = Config::new(token, Intents::GUILD_MESSAGES);
    let mut manager = discord::manager::connect(config).await?;

    while let Ok(event) = manager.recv().await {
        if let Ok(event) = DispatchEvent::try_from(event) {
            handle_event(&mut manager, event).await;
        }
    }

    Ok(())
}

async fn handle_event(manager: &mut Manager, event: DispatchEvent) {
    let rest = manager.rest();
    if let DispatchEvent::MessageCreate(msg) = event {
        if msg.content.starts_with("!ping") {
            let _ = rest
                .create_message(msg.channel_id)
                .reply(msg.id)
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
        if msg.content.starts_with("!reconnect") {
            manager
                .context_mut()
                .recv(&twilight_model::gateway::event::GatewayEvent::Reconnect);
        }
    }
}
