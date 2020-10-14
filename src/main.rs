extern crate async_std;
extern crate chrono;
extern crate discord;
extern crate log;
extern crate serde_json;
extern crate simple_logger;
extern crate tokio;

use discord::model::User;
use discord::rest::RestClient;
use discord::wrapper::Wrap;
use discord::Event;
use futures::StreamExt;
use std::sync::{Arc, RwLock};
use tokio::runtime::Runtime;

#[derive(Debug)]
struct State {
    running_since: std::time::Instant,
    current_user: User,
}

fn main() {
    let token = "NTEyMzAxMjI4ODAyNDQxMjI2.W-xLsw.gXVtWaEmOJ1ZhiL-20cuG4vYxHw";
    // env_logger::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Debug)
        .init()
        .unwrap();

    let mut rt = Runtime::new().unwrap();

    let handle = rt.handle().clone();
    let rest_client = Arc::new(RestClient::new(token));

    let mut state = None;

    rt.block_on(async {
        let shard = discord::gateway::Shard::with_rest_client(token, Arc::clone(&rest_client));
        let (conn, mut events) = shard.connection();

        let running = handle.spawn(conn.run());

        while let Some(event) = events.next().await {
            match event {
                Event::Ready(current_user) => {
                    state = Some(Arc::new(RwLock::new(State {
                        running_since: std::time::Instant::now(),
                        current_user,
                    })));
                }
                Event::MessageCreate(msg) => {
                    let msg = msg.wrap(Arc::clone(&rest_client));
                    handle.spawn(handle_message(msg, Arc::clone(state.as_ref().unwrap())));
                }
                Event::MessageUpdate(msg) => {
                    dbg!(msg);
                }
                Event::MessageDelete(deleted) => {
                    dbg!(deleted);
                }
                Event::GuildCreate(_) => {}
                _ => {
                    // println!("{}", serde_json::to_string_pretty(&event).unwrap());
                }
            }
        }

        let _ = running.await.unwrap();
    });
}

async fn handle_message(msg: discord::model::MessageWrapper, state: Arc<RwLock<State>>) {
    if msg.author.bot {
        return;
    }

    match msg.content.as_str() {
        "ping" => {
            msg.reply("pong").await.unwrap();
        }
        "uptime" => {
            msg.reply(format!(
                "{}m",
                state.read().unwrap().running_since.elapsed().as_secs() / 60
            ))
            .await
            .unwrap();
        }
        "delete" => {
            msg.delete().await.unwrap();
        }
        _ => {
            // msg.reply(&msg.content).await.unwrap();
        }
    };
}

// fn main_test() {
//     let builder = ShardBuilder::new("token");

//     let (conn, shard) = builder.build()

//     tokio::spawn(conn);

//     let events = shard.events();

//     while let Some(event) = events.recv().await {
//     }

// }
