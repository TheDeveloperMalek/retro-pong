

use super::ClientEvent;
use crate::network::protocol::{ClientMessage, ServerMessage};
use std::{
    io::{self, Read, Write},
    net::{SocketAddr, TcpStream, ToSocketAddrs, UdpSocket},
    sync::mpsc::{Receiver, Sender, TryRecvError},
    thread,
    time::{Duration, Instant},
};

pub(super) fn normalize_endpoint(endpoint: &str) -> String {
    let endpoint = endpoint.trim().trim_start_matches("tcp://");
    if endpoint.is_empty() { return "127.0.0.1:41123".into(); }
    
    if endpoint.starts_with('[') && endpoint.ends_with(']') { return format!("{endpoint}:41123"); }
    if !endpoint.contains(':') { format!("{endpoint}:41123") } else { endpoint.to_string() }
}

pub(super) fn local_lan_ipv4() -> String {
    
    let Ok(socket) = UdpSocket::bind("0.0.0.0:0") else { return "127.0.0.1".into(); };
    if socket.connect("1.1.1.1:80").is_ok() {
        if let Ok(address) = socket.local_addr() {
            if !address.ip().is_loopback() { return address.ip().to_string(); }
        }
    }
    "127.0.0.1".into()
}

pub(super) fn connection_worker(
    endpoint: String,
    player_name: String,
    initial_token: Option<String>,
    outbound: Receiver<ClientMessage>,
    events: Sender<ClientEvent>,
) {
    let mut reconnect_token = initial_token;
    loop {
        let address = match resolve_endpoint(&endpoint) {
            Ok(address) => address,
            Err(error) => {
                let _ = events.send(ClientEvent::Disconnected(error));
                thread::sleep(Duration::from_secs(2));
                continue;
            }
        };
        match TcpStream::connect_timeout(&address, Duration::from_secs(5)) {
            Ok(mut writer) => {
                let _ = writer.set_write_timeout(Some(Duration::from_millis(250)));
                let mut reader = match writer.try_clone() {
                    Ok(stream) => stream,
                    Err(error) => {
                        let _ = events.send(ClientEvent::Disconnected(error.to_string()));
                        thread::sleep(Duration::from_secs(2));
                        continue;
                    }
                };
                let _ = reader.set_read_timeout(Some(Duration::from_millis(50)));
                if send_packet(&mut writer, &ClientMessage::Hello {
                    player_name: player_name.clone(), reconnect_token: reconnect_token.clone(),
                }).is_err() {
                    let _ = events.send(ClientEvent::Disconnected("Could not send handshake".into()));
                    thread::sleep(Duration::from_secs(2));
                    continue;
                }
                let _ = events.send(ClientEvent::Connected);
                
                
                
                let mut incoming = Vec::<u8>::with_capacity(4096);
                let mut chunk = [0_u8; 4096];
                let mut last_ping = Instant::now();
                let mut last_server_packet = Instant::now();
                let mut ping_nonce = 0_u64;
                'connection: loop {
                    loop {
                        match outbound.try_recv() {
                            Ok(message) => {
                                if send_packet(&mut writer, &message).is_err() {
                                    break 'connection;
                                }
                            }
                            Err(TryRecvError::Empty) => break,
                            Err(TryRecvError::Disconnected) => return,
                        }
                    }

                    if last_server_packet.elapsed() >= Duration::from_secs(25) {
                        let _ = events.send(ClientEvent::Disconnected("Server heartbeat timed out".into()));
                        break 'connection;
                    }

                    if last_ping.elapsed() >= Duration::from_secs(10) {
                        ping_nonce = ping_nonce.wrapping_add(1);
                        if send_packet(&mut writer, &ClientMessage::Ping { nonce: ping_nonce }).is_err() {
                            break 'connection;
                        }
                        last_ping = Instant::now();
                    }

                    match reader.read(&mut chunk) {
                        Ok(0) => break 'connection,
                        Ok(bytes_read) => {
                            last_server_packet = Instant::now();
                            incoming.extend_from_slice(&chunk[..bytes_read]);
                            match take_complete_lines(&mut incoming) {
                                Ok(packets) => for packet in packets {
                                    match serde_json::from_slice::<ServerMessage>(&packet) {
                                        Ok(message) => {
                                            if let ServerMessage::Welcome { reconnect_token: token, .. } = &message {
                                                reconnect_token = Some(token.clone());
                                            }
                                            if events.send(ClientEvent::Message(message)).is_err() { return; }
                                        }
                                        Err(error) => eprintln!("Invalid server packet: {error}"),
                                    }
                                },
                                Err(()) => {
                                    let _ = events.send(ClientEvent::Disconnected("Server packet exceeded 64 KiB".into()));
                                    break 'connection;
                                }
                            }
                        }
                        Err(error) if matches!(error.kind(), io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut | io::ErrorKind::Interrupted) => {}
                        Err(error) => {
                            let _ = events.send(ClientEvent::Disconnected(error.to_string()));
                            break 'connection;
                        }
                    }
                    thread::sleep(Duration::from_millis(4));
                }
                let _ = events.send(ClientEvent::Disconnected("Connection lost; attempting reconnection".into()));
            }
            Err(error) => {
                let _ = events.send(ClientEvent::Disconnected(format!("Connection failed: {error}")));
            }
        }
        thread::sleep(Duration::from_secs(2));
    }
}

pub(super) const MAX_SERVER_PACKET_BYTES: usize = 65_536;



pub(super) fn take_complete_lines(buffer: &mut Vec<u8>) -> Result<Vec<Vec<u8>>, ()> {
    let mut packets = Vec::new();
    while let Some(newline) = buffer.iter().position(|byte| *byte == b'\n') {
        if newline > MAX_SERVER_PACKET_BYTES { return Err(()); }
        let mut frame: Vec<u8> = buffer.drain(..=newline).collect();
        frame.pop(); 
        if frame.last() == Some(&b'\r') { frame.pop(); }
        packets.push(frame);
    }
    if buffer.len() > MAX_SERVER_PACKET_BYTES { return Err(()); }
    Ok(packets)
}

pub(super) fn resolve_endpoint(endpoint: &str) -> Result<SocketAddr, String> {
    let address = normalize_endpoint(endpoint);
    address.to_socket_addrs().map_err(|error| error.to_string())?
        .next().ok_or_else(|| "The server address could not be resolved".to_string())
}

pub(super) fn send_packet(stream: &mut TcpStream, message: &ClientMessage) -> io::Result<()> {
    let mut bytes = serde_json::to_vec(message).map_err(io::Error::other)?;
    bytes.push(b'\n');
    stream.write_all(&bytes)?;
    stream.flush()
}

pub fn discover_lan_servers(timeout: Duration) -> Vec<String> {
    let Ok(socket) = UdpSocket::bind("0.0.0.0:0") else { return Vec::new(); };
    let _ = socket.set_broadcast(true);
    let _ = socket.set_read_timeout(Some(timeout));
    let _ = socket.send_to(b"RETRO_PONG_DISCOVER", format!("255.255.255.255:{}", 41_124));
    let mut servers = Vec::new();
    let mut buffer = [0_u8; 512];
    while let Ok((length, _)) = socket.recv_from(&mut buffer) {
        if let Ok(message) = std::str::from_utf8(&buffer[..length]) {
            let parts: Vec<&str> = message.split('|').collect();
            if parts.first() == Some(&"RETRO_PONG_SERVER") {
                if let Some(address) = parts.get(1) { servers.push((*address).to_string()); }
            }
        }
    }
    servers.sort();
    servers.dedup();
    servers
}

