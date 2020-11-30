#![allow(dead_code)]
extern crate async_std;
extern crate chrono;
extern crate discord;
extern crate log;
extern crate serde_json;
extern crate simple_logger;
extern crate tokio;

use discord::cache::Cache;
use discord::model::gateway::DispatchEvent;
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
    let token = "NTEyMzAxMjI4ODAyNDQxMjI2.W-xLsw.gXVtWaEmOJ1ZhiL-20cuG4vYxHw";
    // env_logger::from_env(env_logger::Env::default().default_filter_or("debug")).init();
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

    let shard = discord::gateway::Shard::with_rest_client(token, rest_client.clone());
    let (conn, mut events) = shard.connection();

    let running = tokio::spawn(conn.run());

    while let Some(event) = events.next().await {
        cache.update(&event);

        tokio::spawn(handle_event(event, Arc::clone(&state), rest_client.clone()));
    }

    let _ = running.await.unwrap();

    Ok(())
}

async fn handle_event(event: DispatchEvent, state: Arc<RwLock<State>>, rest_client: RestClient) {
    // dbg!(&event);
    // match &event {
    //     DispatchEvent::GuildMemberAdd(_)
    //     | DispatchEvent::GuildMemberRemove(_)
    //     | DispatchEvent::GuildMemberUpdate(_) => {
    //         dbg!(&event);
    //     }
    //     _ => {}
    // }
    // let mut unnamed = std::fs::OpenOptions::new()
    //     .append(true)
    //     .create(true)
    //     .open("unnamed.mpak")
    //     .unwrap();
    // let mut named = std::fs::OpenOptions::new()
    //     .append(true)
    //     .create(true)
    //     .open("named.mpak")
    //     .unwrap();

    // rmp_serde::encode::write(&mut unnamed, &event).unwrap();
    // rmp_serde::encode::write_named(&mut named, &event).unwrap();

    match event {
        DispatchEvent::Ready(ready) => {
            let mut state = state.write().unwrap();
            state.current_user = Some(ready.user);
            state.running_since = Some(std::time::Instant::now());
        }
        DispatchEvent::MessageCreate(msg) => {
            let msg = rest_client.wrap(msg);
            handle_message(msg, state).await;
        }
        DispatchEvent::MessageUpdate(msg) => {
            dbg!(msg);
        }
        DispatchEvent::MessageDelete(deleted) => {
            dbg!(deleted);
        }
        DispatchEvent::Resume => {
            dbg!("resumed");
        }
        _ => {
            // println!("{}", serde_json::to_string_pretty(&event).unwrap());
        }
    }
}

async fn handle_message(msg: discord::model::MessageWrapper, state: Arc<RwLock<State>>) {
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
            // msg.reply(&msg.content).await.unwrap();
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

    // match max_part {
    //     DurationPart::Second => (0, 0, 0, secs_total),
    //     DurationPart::Minute => (0, 0, minutes_total, secs),
    //     DurationPart::Hour => (0, hours_total, minutes, secs),
    //     DurationPart::Day => (days, hours, minutes, secs),
    // }
}
