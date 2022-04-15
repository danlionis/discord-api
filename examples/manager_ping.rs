use discord::{proto::Config, Error};
use std::{convert::TryFrom, sync::Arc};
use twilight_http::{request::channel::reaction::RequestReactionType, Client};
use twilight_model::gateway::{
    event::DispatchEvent,
    payload::outgoing::update_presence::UpdatePresencePayload,
    presence::{ActivityType, MinimalActivity, Status},
    Intents,
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let token = std::env::var("TOKEN").expect("missing TOKEN env variable");

    env_logger::init();

    let config = Config::new(token, Intents::GUILD_MESSAGES).presence(
        UpdatePresencePayload::new(
            vec![MinimalActivity {
                kind: ActivityType::Playing,
                name: "Hello World".to_string(),
                url: None,
            }
            .into()],
            false,
            None,
            Status::Online,
        )
        .unwrap(),
    );
    let mut manager = discord::manager::connect(config).await?;

    while let Ok(event) = manager.recv().await {
        if let Ok(event) = DispatchEvent::try_from(event) {
            let rest = Arc::clone(manager.rest());
            tokio::spawn(handle_event(rest, event));
        }
    }

    Ok(())
}

async fn handle_event(rest: Arc<Client>, event: DispatchEvent) {
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
    }
}
