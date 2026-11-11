

use std::{
    net::{SocketAddr, UdpSocket},
    thread,
};

const DISCOVERY_PORT: u16 = 41_124;
const DISCOVERY_REQUEST: &[u8] = b"RETRO_PONG_DISCOVER";
const DISCOVERY_RESPONSE_PREFIX: &str = "RETRO_PONG_SERVER";

pub(super) fn spawn_discovery_responder(server_addr: String) {
    thread::spawn(move || {
        let socket = match UdpSocket::bind(format!("0.0.0.0:{DISCOVERY_PORT}")) {
            Ok(socket) => socket,
            Err(error) => {
                eprintln!("LAN discovery unavailable: {error}");
                return;
            }
        };

        let mut buffer = [0_u8; 256];
        loop {
            let Ok((length, sender)) = socket.recv_from(&mut buffer) else {
                continue;
            };
            if &buffer[..length] != DISCOVERY_REQUEST {
                continue;
            }

            let advertised_address = discover_reachable_address(&server_addr, sender)
                .unwrap_or_else(|| server_addr.clone());
            let response = format!("{DISCOVERY_RESPONSE_PREFIX}|{advertised_address}|1");
            let _ = socket.send_to(response.as_bytes(), sender);
        }
    });
}

fn discover_reachable_address(server_addr: &str, recipient: SocketAddr) -> Option<String> {
    let bound_address = server_addr.parse::<SocketAddr>().ok()?;
    let route = UdpSocket::bind("0.0.0.0:0").ok()?;
    route.connect(recipient).ok()?;
    let local_ip = route.local_addr().ok()?.ip();

    Some(SocketAddr::new(local_ip, bound_address.port()).to_string())
}
