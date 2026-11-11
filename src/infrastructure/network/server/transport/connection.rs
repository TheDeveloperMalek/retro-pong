

use crate::network::protocol::{ClientMessage, ServerMessage};
use super::ServerEvent;
use std::{
    io::{self, BufRead, BufReader, Write},
    net::TcpStream,
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

const MAX_CLIENT_PACKET_BYTES: usize = 65_536;

pub(super) fn spawn_connection(
    stream: TcpStream,
    connection_id: u64,
    event_sender: Sender<ServerEvent>,
) {
    thread::spawn(move || {
        let _ = stream.set_nodelay(true);

        let writer = match stream.try_clone() {
            Ok(writer) => {
                let _ = writer.set_nodelay(true);
                writer
            }
            Err(error) => {
                eprintln!("Could not clone client socket: {error}");
                return;
            }
        };

        let (outbound_sender, outbound_receiver) = mpsc::channel::<ServerMessage>();
        let writer_thread = thread::spawn(move || write_messages(writer, outbound_receiver));

        if event_sender
            .send(ServerEvent::Connected {
                connection_id,
                outbound: outbound_sender,
            })
            .is_err()
        {
            return;
        }

        read_client_messages(stream, connection_id, &event_sender);
        let _ = event_sender.send(ServerEvent::Disconnected { connection_id });
        let _ = writer_thread.join();
    });
}

fn read_client_messages(
    stream: TcpStream,
    connection_id: u64,
    event_sender: &Sender<ServerEvent>,
) {
    let mut reader = BufReader::new(stream);
    let mut packet = Vec::with_capacity(4096);

    loop {
        match read_packet(&mut reader, &mut packet) {
            Ok(true) => match serde_json::from_slice::<ClientMessage>(&packet) {
                Ok(message) => {
                    if event_sender
                        .send(ServerEvent::Message {
                            connection_id,
                            message,
                        })
                        .is_err()
                    {
                        break;
                    }
                }
                Err(error) => eprintln!("Invalid client packet: {error}"),
            },
            Ok(false) => break,
            Err(error) => {
                eprintln!("Client {connection_id} closed: {error}");
                break;
            }
        }
    }
}


fn read_packet(reader: &mut BufReader<TcpStream>, packet: &mut Vec<u8>) -> io::Result<bool> {
    packet.clear();

    loop {
        let available = reader.fill_buf()?;
        if available.is_empty() {
            return Ok(false);
        }

        let newline = available.iter().position(|byte| *byte == b'\n');
        let bytes_to_consume = newline.map_or(available.len(), |index| index + 1);
        if packet.len().saturating_add(bytes_to_consume) > MAX_CLIENT_PACKET_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "client packet exceeded the 64 KiB limit",
            ));
        }

        packet.extend_from_slice(&available[..bytes_to_consume]);
        reader.consume(bytes_to_consume);

        if newline.is_some() {
            packet.pop(); 
            if packet.last() == Some(&b'\r') {
                packet.pop();
            }
            return Ok(true);
        }
    }
}

fn write_messages(mut stream: TcpStream, receiver: Receiver<ServerMessage>) {
    while let Ok(message) = receiver.recv() {
        let Ok(mut packet) = serde_json::to_vec(&message) else {
            continue;
        };
        packet.push(b'\n');

        if stream.write_all(&packet).is_err() || stream.flush().is_err() {
            break;
        }
    }
}

