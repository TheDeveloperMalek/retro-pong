#[cfg(not(target_arch = "wasm32"))]
#[path = "../infrastructure/network/protocol.rs"]
pub mod protocol;

#[cfg(not(target_arch = "wasm32"))]
#[path = "../infrastructure/network/server.rs"]
pub mod server;

#[cfg(not(target_arch = "wasm32"))]
#[path = "../infrastructure/network/client.rs"]
pub mod client;
