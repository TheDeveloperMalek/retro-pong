# Retro Pong — Arcade Edition

A modern take on classic arcade Pong. Play solo against four AI difficulty levels, challenge a friend locally, collect tactical power-ups, and enjoy a modern arcade experience.

## Supported Platforms

- **Windows:** Native desktop application.
- **Linux:** Native desktop application.
- **macOS:** Native desktop application.
- **Web:** Play directly in a modern browser using WebAssembly.

**Note:** Network Multiplayer is currently available in the desktop version only.

## Requirements

### Desktop

- Rust 1.97.1 or newer.
- Cargo (included with Rust).
- A compatible graphics and audio environment.
- Internet connection for downloading dependencies during the initial build.

### Web

- Rust 1.97.1 or newer.
- Cargo and Rustup.
- Python 3 for local preview.
- Internet connection for downloading browser build dependencies and required assets.
- A modern browser with JavaScript and WebAssembly enabled.
- WebGL2 support (preferred) or WebGPU support with hardware acceleration.

The build scripts automatically install the required WebAssembly target and matching `wasm-bindgen-cli` version when needed.

## Download & Run — Desktop

Download the project ZIP and extract it.

**Windows**

Double-click:

`Run-Retro-Pong.bat`

**Linux / macOS**

Open a terminal inside the project directory and run:

```bash
chmod +x ./run-retro-pong.sh
./run-retro-pong.sh
```

The launcher automatically prepares, builds, and starts the game.

The first launch may take longer while dependencies are downloaded and the application is compiled. Subsequent launches reuse the local build cache.

**Note:** The ZIP contains the project source code and launch scripts, not prebuilt desktop application binaries.

## Build & Run — Web

The project includes separate scripts for building and locally previewing the browser version.

**Linux / macOS**

Build:

```bash
./scripts/build-web.sh
```

Start the local web server:

```bash
./scripts/serve-web.sh
```

**Windows (PowerShell)**

Build:

```powershell
.\scripts\build-web.ps1
```

Start the local web server:

```powershell
.\scripts\serve-web.ps1
```

Once the server starts, open:

`http://127.0.0.1:8000`

The browser build is generated inside:

`web-dist/`

You can deploy the contents of this directory to a static web hosting service to make the game accessible online.

**Browser requirements:** JavaScript and WebAssembly must be enabled, and the browser must support WebGL2 or WebGPU with hardware acceleration available.

## Multiplayer

Desktop players can host rooms, join friends using room codes, discover games on the same LAN, use quick matchmaking, and play over the Internet.

The desktop multiplayer system also supports player invitations, reconnection, 2v2 teams, and tournaments.

Internet multiplayer requires a reachable host address and an allowed network port through the firewall/router, or an externally hosted server.

The browser version does not currently support network multiplayer.

**Note:** Automatic router configuration, relay services, encrypted transport, and production-grade account security are not included. Use network play on a trusted LAN or a controlled test environment.

## Saved Data

Desktop settings, control bindings, cosmetic selections, match history, and player progress are stored locally in the user's application-data directory.

The browser version uses browser-local storage for settings and player progress.

Save data is normalized when loaded. Unsupported future save formats are kept read-only rather than overwritten.

## License

See [LICENSE](LICENSE).