use discord::{
    manager::{self, http::Client},
    model::{
        gateway::{event::DispatchEvent, Intents},
        id::{
            marker::{ApplicationMarker, GuildMarker},
            Id,
        },
    },
    proto::Config,
    Error,
};
use std::{convert::TryFrom, sync::Arc};
use twilight_model::{
    application::interaction::InteractionType,
    http::interaction::{InteractionResponse, InteractionResponseType},
};
use twilight_util::builder::{command::CommandBuilder, InteractionResponseDataBuilder};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let token = std::env::args().skip(1).next().expect("missing token");

    env_logger::init();

    let config = Config::new(token, Intents::GUILD_MESSAGES);
    let mut manager = manager::connect(config).await?;

    let rest = manager.rest();

    let application_id = {
        let response = rest.current_user_application().await?;

        response.model().await.unwrap().id
    };

    while let Ok(event) = manager.recv().await {
        if let Ok(event) = DispatchEvent::try_from(event) {
            let rest = Arc::clone(manager.rest());
            tokio::spawn(handle_event(rest, event, application_id));
        }
    }

    Ok(())
}

async fn deploy_guild_commands(
    rest: Arc<Client>,
    app_id: Id<ApplicationMarker>,
    guild_id: Id<GuildMarker>,
) -> Result<(), Error> {
    let command = CommandBuilder::new(
        "ping",
        "pong",
        twilight_model::application::command::CommandType::ChatInput,
    )
    .guild_id(guild_id)
    .build();

    rest.interaction(app_id)
        .set_guild_commands(guild_id, &[command])
        .await?;

    Ok(())
}

async fn handle_event(rest: Arc<Client>, event: DispatchEvent, app_id: Id<ApplicationMarker>) {
    match event {
        DispatchEvent::MessageCreate(msg) => {
            if msg.content == "!deploy" {
                deploy_guild_commands(rest, app_id, msg.guild_id.unwrap())
                    .await
                    .unwrap();
            }
        }
        DispatchEvent::InteractionCreate(interaction) => {
            if interaction.kind == InteractionType::ApplicationCommand {
                let response = &InteractionResponse {
                    kind: InteractionResponseType::ChannelMessageWithSource,
                    data: Some(
                        InteractionResponseDataBuilder::new()
                            .content("pong")
                            .build(),
                    ),
                };

                rest.interaction(interaction.application_id)
                    .create_response(interaction.id, &interaction.token, response)
                    .await
                    .unwrap();
            }
        }
        _ => {}
    }
}
