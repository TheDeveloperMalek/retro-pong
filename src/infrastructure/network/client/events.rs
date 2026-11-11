

use super::{ClientEvent, NetworkRuntime};
use crate::{
    game::{Screen, components::{MatchMode, MatchPhase, MatchSession}},
    localization::tr,
    network::protocol::{ClientMessage, PlayerSummary, RoomMode, RoomSummary, ServerMessage},
    persistence::Settings,
};
use bevy::{log::{info, warn}, prelude::{NextState, Res, ResMut}};
use std::sync::mpsc::TryRecvError;

pub fn poll_events(
    mut runtime: ResMut<NetworkRuntime>,
    settings: Res<Settings>,
    mut session: ResMut<MatchSession>,
    mut next_state: ResMut<NextState<Screen>>,
) {
    let mut received = Vec::new();
    if let Ok(receiver) = runtime.events.lock() {
        loop {
            match receiver.try_recv() {
                Ok(event) => received.push(event),
                Err(TryRecvError::Empty | TryRecvError::Disconnected) => break,
            }
        }
    }
    for event in received {
        match event {
            ClientEvent::Connected => {
                runtime.connected = true;
                runtime.status = tr(settings.language, "CONNECTED");
            }
            ClientEvent::Disconnected(reason) => {
                runtime.connected = false;
                warn!("NETWORK DISCONNECTED: {reason}");
                runtime.status = format!("{}: {}", tr(settings.language, "DISCONNECTED"), tr(settings.language, "CONNECTION FAILED CHECK ADDRESS"));
            }
            ClientEvent::DiscoveryCompleted { endpoint, player_name } => {
                if let Some(endpoint) = endpoint {
                    runtime.status = format!("{} {endpoint} // {}...", tr(settings.language, "FOUND SERVER"), tr(settings.language, "JOINING SERVER"));
                    runtime.connect(endpoint, player_name, ClientMessage::QuickMatch { mode: RoomMode::Classic });
    } else {
                    runtime.status = tr(settings.language, "NO GAME FOUND ON THIS LAN // HOST A GAME OR ENTER AN ADDRESS");
                }
            }
            ClientEvent::ServerStartFailed(reason) => {
                runtime.embedded_server_started = false;
                warn!("SERVER START FAILED: {reason}");
                runtime.status = tr(settings.language, "SERVER START FAILED");
            }
            ClientEvent::Message(message) => match message {
                ServerMessage::Welcome { player_id, reconnect_token } => {
                    runtime.status = format!("{} // PLAYER {player_id}", tr(settings.language, "CONNECTED"));
                    runtime.player_id = Some(player_id);
                    runtime.reconnect_token = Some(reconnect_token);
                    if let Some(action) = runtime.pending_action.take() { runtime.send(action); }
                }
                ServerMessage::QueueStatus { queued, position } => {
                    runtime.status = if queued { format!("{} // {} {position}", tr(settings.language, "MATCHMAKING QUEUE"), tr(settings.language, "POSITION")) } else { tr(settings.language, "MATCHMAKING CANCELLED") };
                }
                ServerMessage::RoomList { rooms } => {
                    runtime.status = format!("{} {}", rooms.len(), tr(settings.language, "PUBLIC ROOMS FOUND"));
                    for room in &rooms {
                        info!("ROOM {} // {:?} // {}/{} // HOST {}", room.room_code, room.mode, room.players.len(), room.max_players, room.host_name);
                    }
                    runtime.rooms = rooms;
                }
                ServerMessage::PlayerList { players } => {
                    runtime.status = format!("{} {}", players.len(), tr(settings.language, "OTHER PLAYERS ONLINE"));
                    runtime.players = players.clone();
                    for player in &players {
                        info!("PLAYER {} // {} // TEAM {} // {}", player.player_id, player.display_name, player.team + 1,
                            if player.connected { "ONLINE" } else { "OFFLINE" });
                    }
                }
                ServerMessage::RoomJoined { room, player_id, team, host, invite_code } => {
                    runtime.pending_invite = None;
                    runtime.room_code = Some(room.room_code.clone());
                    runtime.mode = room.mode;
                    runtime.team = team;
                    runtime.host = host;
                    if let Some(code) = invite_code { runtime.invite_code = Some(code.clone()); info!("FRIEND INVITE CODE: {code}"); }
                    let mode = match room.mode { RoomMode::Classic => "CLASSIC", RoomMode::Teams => "TEAM", RoomMode::Tournament => "TOURNAMENT" };
                    runtime.status = format!("{} {} // {} {}", tr(settings.language, "ROOM"), room.room_code, tr(settings.language, mode), team + 1);
                    runtime.players = room.players.clone();
                    upsert_room(&mut runtime.rooms, room.clone());
                    info!("JOINED ROOM {} AS {} // TEAM {}", room.room_code, player_id, team + 1);
                }
                ServerMessage::InviteCreated { invite_code, room_code } => {
                    runtime.invite_code = Some(invite_code.clone());
                    runtime.status = format!("{} {invite_code} // {} {room_code}", tr(settings.language, "INVITE"), tr(settings.language, "ROOM"));
                    info!("SHARE THIS INVITE CODE: {invite_code}");
                }
                ServerMessage::InviteReceived { from_name, invite_code, room_code } => {
                    runtime.pending_invite = Some(invite_code.clone());
                    runtime.status = format!("{} {from_name} // {} {room_code}", tr(settings.language, "INVITATION RECEIVED"), tr(settings.language, "ROOM"));
                    info!("INVITATION RECEIVED FROM {from_name}; PRESS F3 TO JOIN; CODE {invite_code}");
                }
                ServerMessage::PlayerJoined { player } => {
                    let room_code = runtime.room_code.clone();
                    upsert_player(&mut runtime.players, player.clone());
                    if let Some(room_code) = room_code.as_deref() {
                        upsert_room_player(&mut runtime.rooms, room_code, player.clone());
                    }
                    runtime.status = format!("{}: {}", tr(settings.language, "PLAYER JOINED"), player.display_name);
                    info!("PLAYER JOINED: {} // TEAM {}", player.display_name, player.team + 1);
                }
                ServerMessage::PlayerLeft { player_id, reconnect_grace_secs } => {
                    set_player_connection(&mut runtime.players, &player_id, false);
                    let room_code = runtime.room_code.clone();
                    if let Some(room_code) = room_code.as_deref() {
                        set_room_player_connection(&mut runtime.rooms, room_code, &player_id, false);
                    }
                    runtime.status = format!("{player_id} // {} // {reconnect_grace_secs}s", tr(settings.language, "PLAYER DISCONNECTED // AI TAKEOVER"));
                    warn!("PLAYER {player_id} DISCONNECTED; server retains the seat for {reconnect_grace_secs}s");
                }
                ServerMessage::PlayerReconnected { player_id } => {
                    set_player_connection(&mut runtime.players, &player_id, true);
                    let room_code = runtime.room_code.clone();
                    if let Some(room_code) = room_code.as_deref() {
                        set_room_player_connection(&mut runtime.rooms, room_code, &player_id, true);
                    }
                    runtime.status = format!("{player_id} {}", tr(settings.language, "RECONNECTED"));
                    info!("PLAYER {player_id} RECONNECTED; control returned from AI");
                }
                ServerMessage::MatchStarting { snapshot } => {
                    runtime.last_powerup_event_id = snapshot.powerup_events.last().map_or(0, |event| event.id);
                    runtime.latest_snapshot = Some(snapshot);
                    runtime.match_started = true;
                    runtime.local_pause_active = false;
                    runtime.status = format!("{} // {}", tr(settings.language, "MATCH STARTED"), tr(settings.language, "SERVER AUTHORITATIVE"));
                    session.mode = MatchMode::Networked;
                    session.restart_requested = true;
                    next_state.set(Screen::Playing);
                }
                ServerMessage::Snapshot { snapshot } => runtime.latest_snapshot = Some(snapshot),
                ServerMessage::MatchEnded { winning_team, tournament_champion } => {
                    runtime.latest_snapshot = None;
                    runtime.match_started = false;
                    runtime.local_pause_active = false;
                    runtime.status = tournament_champion.map(|name| format!("{}: {name}", tr(settings.language, "TOURNAMENT CHAMPION")))
                        .unwrap_or_else(|| format!("{} {}", tr(settings.language, "TEAM"), winning_team + 1));
                    session.phase = MatchPhase::Finished;
                    session.phase_timer = 0.1;
                    next_state.set(Screen::Results);
                }
                ServerMessage::TournamentUpdate { round, match_index, participants, champion } => {
                    let names = participants.join(" vs ");
                    runtime.status = champion.map(|name| format!("{}: {name}", tr(settings.language, "CHAMPION")))
                        .unwrap_or_else(|| format!("{} // {round} // {match_index} // {names}", tr(settings.language, "TOURNAMENT")));
                    info!("{}", runtime.status);
                }
                ServerMessage::Pong { .. } => {}
                ServerMessage::Error { code, message } => {
                    let error_key = match code.as_str() {
                        "already_in_room" => "ERROR ALREADY IN ROOM",
                        "room_not_found" => "ERROR ROOM NOT FOUND",
                        "private_room" => "ERROR PRIVATE ROOM",
                        "match_in_progress" => "ERROR MATCH IN PROGRESS",
                        "room_full" => "ERROR ROOM FULL",
                        "host_only" => "ERROR HOST ONLY",
                        "already_started" => "ERROR ALREADY STARTED",
                        "tournament_start" => "ERROR TOURNAMENT START",
                        "matchmaking_stale" => "ERROR MATCHMAKING STALE",
                        "not_in_room" => "ERROR NOT IN ROOM",
                        "player_unavailable" => "ERROR PLAYER UNAVAILABLE",
                        "invalid_bracket_size" => "ERROR INVALID BRACKET SIZE",
                        "not_enough_players" => "ERROR NOT ENOUGH PLAYERS",
                        _ => "CONNECTION FAILED CHECK ADDRESS",
                    };
                    runtime.status = format!("{}: {}", tr(settings.language, "NETWORK ERROR"), tr(settings.language, error_key));
                    warn!("NETWORK ERROR {code}: {message}");
                }
            },
        }
    }
}



fn upsert_player(players: &mut Vec<PlayerSummary>, player: PlayerSummary) {
    if let Some(existing) = players.iter_mut().find(|existing| existing.player_id == player.player_id) {
        *existing = player;
    } else {
        players.push(player);
    }
}

fn set_player_connection(players: &mut [PlayerSummary], player_id: &str, connected: bool) {
    if let Some(player) = players.iter_mut().find(|player| player.player_id == player_id) {
        player.connected = connected;
    }
}

fn upsert_room(rooms: &mut Vec<RoomSummary>, room: RoomSummary) {
    if let Some(existing) = rooms.iter_mut().find(|existing| existing.room_code == room.room_code) {
        *existing = room;
    } else {
        rooms.push(room);
    }
}

fn upsert_room_player(rooms: &mut [RoomSummary], room_code: &str, player: PlayerSummary) {
    if let Some(room) = rooms.iter_mut().find(|room| room.room_code == room_code) {
        upsert_player(&mut room.players, player);
    }
}

fn set_room_player_connection(rooms: &mut [RoomSummary], room_code: &str, player_id: &str, connected: bool) {
    if let Some(room) = rooms.iter_mut().find(|room| room.room_code == room_code) {
        set_player_connection(&mut room.players, player_id, connected);
    }
}

