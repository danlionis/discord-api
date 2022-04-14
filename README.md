# Discord Library

STILL UNDER DEVELOPMENT!
LOOKING FOR A GOOD NAME!

Low level implementation of the discord gateway protocol.
The library handles the state of the gateway connection by processing incoming messages and generating outgoing messages.

The application is responsible to provide the actual I/O (e.g. websockets, event loop). 
This allows the library to be completely runtime agnostic (even working with synchronous sockets).

The library also provides a basic managed connection (behind the `manager` feature flag).
This Manager uses [tokio](https://github.com/tokio-rs/tokio) and [tokio\_tungstenite](https://github.com/snapview/tokio-tungstenite) as its I/O stack.
This is probably the best choice for most useres if you are looking for the easiest way to get your bot running.

This protocol handling is inspired by Cloudflares [quiche](https://github.com/cloudflare/quiche) pattern of handling I/O and state.

# Getting started

## Examples
The repo provides two examples:

A basic ping example that directly interacts with the connection by forwarding incoming packets:
```bash
$ cargo run --example ping --all-features
```
and a managed connection that also handles the io
```bash
$ cargo run --example manager_ping --all-features
```
Both examples require the `TOKEN` environment variable to be set to your bot token

## Connecting
The first step in establishing a connection is to create a Connection object with you login token:
```rust
let conn = Connection::new();
```
Since the Connection doesn't handle I/O by itself it stays in a closed state until it receives the correct packets from the gateway.

## Handling incoming events
The connection processes incoming events with its `recv()` method.
In case the websocket connection closes, the close code can also be forwarded to the Connection for it to handle a potential reconnect:
```rust
loop {
    match websocket.recv() {
        Ok(gateway_event) => {
            // handle event...
            conn.recv(gateway_event);ko
        },
        Err(close_code) > {
            // handle websocket closing...
            conn.recv_close_code(close_code);
        }
    }
}
```

## Generating outgoing commands
Outgoing packets are generated with the `send()` or `send_iter()` methods.
These packets have to be sent over the websocket to the gateway.
```rust
for cmd in conn.send_iter() {
    websocket.send(cmd).unwrap();
}
```

## Heartbeating
The application is responsible to maintain a heartbeat timer and queue the corresponding command.

The heartbeat interval can be obtained after the connection is established:
```rust
let heartbeat_interval = conn.heartbeat_interval();
```

To queue a hearbeat command to be sent the next time outgoing commands are generated use:
```rust
conn.queue_heartbeat();
```

The exact implementation details are up to the application, but an implementation can be found in the `ping` example.

## Handling state
There are multiple instances that require an I/O interaction that is not strictly a `send`.
This includes for example that the gateway requested a reconnect.
Below are listed all methods that require special handling by the application:

```rust
if conn.should_reconnect() {
    // reconnect websocket
}

if let Some(code) = conn.failed() {
    // handle a failed and unrecoverable connection (most likely exit the application)
}
```

An example how these cases are handled can also be found in the `ping` example.


