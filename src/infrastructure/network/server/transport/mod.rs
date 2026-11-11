

mod connection;
mod discovery;

use super::hub::Hub;
use crate::network::protocol::*;
use std::{
    io,
    net::TcpListener,
    sync::mpsc::{self, Receiver, Sender, TryRecvError},
    thread,
    time::Duration,
};

use connection::spawn_connection;
use discovery::spawn_discovery_responder;

const DISCOVERY_PORT: u16 = 41_124;

#[derive(Clone, Debug)]
pub struct ServerConfig {
    pub bind_addr: String,
    pub winning_score: u32,
    pub settings: NetworkGameSettings,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_addr: "0.0.0.0:41123".to_string(),
            winning_score: 7,
            settings: NetworkGameSettings::default(),
        }
    }
}

enum ServerEvent {
    Connected {
        connection_id: u64,
        outbound: Sender<ServerMessage>,
    },
    Message {
        connection_id: u64,
        message: ClientMessage,
    },
    Disconnected {
        connection_id: u64,
    },
}



pub fn run_server(config: ServerConfig) -> io::Result<()> {
    let listener = TcpListener::bind(&config.bind_addr)?;
    listener.set_nonblocking(true)?;

    let local_addr = listener.local_addr()?;
    println!("Retro Pong multiplayer service listening on {local_addr}");
    println!("Transport: newline-delimited JSON over TCP; LAN discovery UDP/{DISCOVERY_PORT}");
    spawn_discovery_responder(local_addr.to_string());

    let (event_sender, event_receiver) = mpsc::channel();
    let mut hub = Hub::with_settings(config.winning_score, config.settings);
    let mut next_connection_id = 1_u64;

    loop {
        accept_pending_connections(&listener, &event_sender, &mut next_connection_id);
        process_pending_events(&event_receiver, &mut hub)?;
        hub.tick(1.0 / 30.0);
        thread::sleep(Duration::from_millis(4));
    }
}

fn accept_pending_connections(
    listener: &TcpListener,
    event_sender: &Sender<ServerEvent>,
    next_connection_id: &mut u64,
) {
    loop {
        match listener.accept() {
            Ok((stream, address)) => {
                let connection_id = *next_connection_id;
                *next_connection_id = (*next_connection_id).wrapping_add(1);
                println!("Client connected: {address}");
                spawn_connection(stream, connection_id, event_sender.clone());
            }
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => break,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => {
                eprintln!("Accept error: {error}");
                break;
            }
        }
    }
}

fn process_pending_events(
    event_receiver: &Receiver<ServerEvent>,
    hub: &mut Hub,
) -> io::Result<()> {
    loop {
        match event_receiver.try_recv() {
            Ok(ServerEvent::Connected { connection_id, outbound }) => {
                hub.on_connected(connection_id, outbound);
            }
            Ok(ServerEvent::Message { connection_id, message }) => {
                hub.handle(connection_id, message);
            }
            Ok(ServerEvent::Disconnected { connection_id }) => {
                hub.on_disconnected(connection_id);
            }
            Err(TryRecvError::Empty) => return Ok(()),
            Err(TryRecvError::Disconnected) => {
                return Err(io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    "all multiplayer event senders have disconnected",
                ));
            }
        }
    }
}
