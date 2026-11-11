
pub(crate) mod network {
    #[path = "../../src/network/protocol.rs"]
    pub(crate) mod protocol;
    #[path = "../../src/network/server.rs"]
    pub(crate) mod server;
}

use network::server;

fn main() {
    let mut config = server::ServerConfig::default();
    let mut args = std::env::args().skip(1);
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--bind" => {
                if let Some(value) = args.next() { config.bind_addr = value; }
            }
            "--winning-score" => {
                if let Some(value) = args.next() {
                    match value.parse::<u32>() {
                        Ok(score) => config.winning_score = score.clamp(1, 99),
                        Err(_) => {
                            eprintln!("Invalid --winning-score value: {value}");
                            std::process::exit(2);
                        }
                    }
                }
            }
            "--help" | "-h" => {
                println!("Retro Pong Dedicated Server\n\nUsage: retro-pong-server [--bind 0.0.0.0:41123] [--winning-score 7]");
                return;
            }
            other => {
                eprintln!("Unknown argument: {other}. Use --help.");
                std::process::exit(2);
            }
        }
    }
    if let Err(error) = server::run_server(config) {
        eprintln!("Server failed: {error}");
        std::process::exit(1);
    }
}
