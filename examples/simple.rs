#![allow(dead_code)]

use discord::cache::Cache;
use discord::gateway::Shard;
use discord::model::gateway::Event;
use discord::model::User;
use discord::rest::RestClient;
use futures::prelude::*;
use std::sync::{Arc, RwLock};

#[derive(Debug)]
struct State {
    running_since: Option<std::time::Instant>,
    current_user: Option<User>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = std::env::var("TOKEN").expect("missing token");
    let token = token.as_str();
    // let token = "Invalid Token";
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
    let (mut conn, mut events) = shard.connection();

    let running = tokio::spawn(async move { conn.run().await });

    while let Some(event) = events.next().await {
        cache.update(&event);

        tokio::spawn(handle_event(event, Arc::clone(&state), rest_client.clone()));
    }

    let _res = running.await.unwrap().unwrap();

    Ok(())
}

async fn handle_event(event: Event, state: Arc<RwLock<State>>, rest_client: RestClient) {
    match event {
        Event::MessageCreate(msg) => handle_message(rest_client.wrap(*msg), state).await,
        Event::Ready(_) => {
            state.write().unwrap().running_since = Some(std::time::Instant::now());
        }
        _ => {}
    }
}

async fn handle_message(msg: discord::model::MessageWrapper, state: Arc<RwLock<State>>) {
    if msg.author.as_ref().map(|a| a.bot).unwrap_or(false) {
        return;
    }
    dbg!(&msg);

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
        "reply" => {}
        _ => {}
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
