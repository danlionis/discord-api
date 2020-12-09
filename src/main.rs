#![allow(dead_code)]
extern crate async_std;
extern crate chrono;
extern crate discord;
extern crate log;
extern crate serde_json;
extern crate simple_logger;
extern crate tokio;

use discord::cache::Cache;
use discord::gateway::Shard;
use discord::model::gateway::Event;
use discord::model::User;
use discord::rest::RestClient;
use futures::StreamExt;
use std::sync::{Arc, RwLock};

#[derive(Debug)]
struct State {
    running_since: Option<std::time::Instant>,
    current_user: Option<User>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = std::env::var("TOKEN");
    let token = "NTEyMzAxMjI4ODAyNDQxMjI2.W-xLsw.gXVtWaEmOJ1ZhiL-20cuG4vYxHw";
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Debug)
        .init()
        .unwrap();

    let rest_client = RestClient::new(token);

    let state = Arc::new(RwLock::new(State {
        running_since: None,
        current_user: None,
    }));

    let mut cache = Cache::new();

    let shard = Shard::with_rest_client(token, rest_client.clone());
    let (conn, mut events) = shard.connection();

    let running = tokio::spawn(conn.run());

    while let Some(event) = events.next().await {
        cache.update(&event);

        tokio::spawn(handle_event(event, Arc::clone(&state), rest_client.clone()));
    }

    let _x = running.await.unwrap().unwrap();

    Ok(())
}

async fn handle_event(event: Event, _state: Arc<RwLock<State>>, _rest_client: RestClient) {
    // dbg!(&event);

    // if let DispatchEvent::GuildCreate(_) = event {
    //     return;
    // }

    match event {
        Event::MessageCreate(msg) => handle_message(_rest_client.wrap(msg), _state).await,
        _ => {}
    }

    // let ser_event = rmp_serde::to_vec(&event).unwrap();

    // dbg!(&ser_event);

    // let de_event: DispatchEvent = rmp_serde::from_slice(&ser_event).unwrap();

    // rmp_serde::encode::write(&mut unnamed, &event).unwrap();
    // rmp_serde::encode::write_named(&mut named, &event).unwrap();

    // dbg!(de_event);
}

async fn handle_message(msg: discord::model::MessageWrapper, state: Arc<RwLock<State>>) {
    if msg.author.as_ref().map(|a| a.bot).unwrap_or(false) {
        return;
    }

    match msg.content.as_str() {
        "ping" => {
            msg.reply("pong").await.unwrap();
        }
        "uptime" => {
            let uptime = state
                .read()
                .ok()
                .map(|s| s.running_since)
                .flatten()
                .map(|i| i.elapsed())
                .unwrap_or_default();

            let d = duration_parts(uptime);

            msg.reply(format!(
                "```json\n{}```",
                serde_json::to_string_pretty(&d).unwrap()
            ))
            .await
            .unwrap();
        }
        "delete" => {
            msg.delete().await.unwrap();
        }
        _ => {
            dbg!(msg);
        }
    };
}
#[derive(Debug, serde::Serialize)]
struct DurationSplit {
    seconds_total: u64,
    minutes_total: u64,
    hours_total: u64,
    days_total: u64,
}

impl DurationSplit {
    pub fn seconds(&self) -> u64 {
        self.seconds_total % 60
    }

    pub fn minutes(&self) -> u64 {
        self.minutes_total % 60
    }

    pub fn hours(&self) -> u64 {
        self.hours_total % 24
    }

    pub fn days(&self) -> u64 {
        self.days_total
    }
}

// impl std::convert::From<std::time::Duration> for DurationSplit {}

fn duration_parts(duration: std::time::Duration) -> DurationSplit {
    let seconds_total = duration.as_secs();
    let minutes_total = seconds_total / 60;
    let hours_total = minutes_total / 60;
    let days_total = hours_total / 24;

    DurationSplit {
        seconds_total,
        minutes_total,
        hours_total,
        days_total,
    }
}
