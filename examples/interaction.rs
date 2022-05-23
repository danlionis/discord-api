use discord::{
    manager::{self, http::Client},
    model::{
        application::{
            command::{BaseCommandOptionData, CommandOption},
            component::{ActionRow, Component, TextInput},
            interaction::Interaction,
        },
        gateway::{event::DispatchEvent, Intents},
        http::interaction::InteractionResponse,
        id::{
            marker::{ApplicationMarker, GuildMarker},
            Id,
        },
    },
    proto::Config,
    Error,
};
use std::{convert::TryFrom, sync::Arc};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let token = std::env::args().skip(1).next().expect("missing token");

    env_logger::init();

    let config = Config::new(token, Intents::GUILD_MESSAGES);
    let mut manager = manager::connect(config).await?;

    let rest = manager.rest();

    let application_id = {
        let info = rest
            .current_user_application()
            .exec()
            .await?
            .model()
            .await
            .unwrap();
        info.id
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
    let _command = rest
        .interaction(app_id)
        .create_guild_command(guild_id)
        .chat_input("report", "Report user")
        .unwrap()
        .command_options(&[CommandOption::User(BaseCommandOptionData {
            name: "target".to_owned(),
            description: "target channel".to_owned(),
            description_localizations: None,
            name_localizations: None,
            required: true,
        })])
        .unwrap()
        .exec()
        .await?
        .model()
        .await
        .unwrap();
    dbg!(_command);

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
        DispatchEvent::InteractionCreate(interaction) => match &interaction.0 {
            Interaction::ApplicationCommand(application_command) => {
                rest.interaction(app_id).create_response(
                    interaction.id(),
                    &application_command.token,
                    &InteractionResponse {
                        kind: twilight_model::http::interaction::InteractionResponseType::Modal,
                        data: Some(
                            twilight_util::builder::InteractionResponseDataBuilder::new()
                                .title("test".to_owned())
                                .custom_id("report_modal".to_owned())
                                .components([
                                    Component::ActionRow(ActionRow {
                                        components: vec![
                                            Component::TextInput(TextInput {
                                            custom_id: "reason".to_owned(),
                                            required: Some(true),
                                            label: "Reason".to_owned(),
                                            max_length: None,
                                            min_length: None,
                                            placeholder: Some("Why do you want to report".to_owned()),
                                            style: twilight_model::application::component::text_input::TextInputStyle::Paragraph,
                                            value: None,
                                    })
                                    ],
                                    }),
                                ])
                                .build(),
                        ),
                    },
                ).exec().await.unwrap();
            }
            _ => {}
        },
        _ => {}
    }
}
