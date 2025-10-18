# Packet Visualizer in Rust

## Requirements

- Rust stable
- Npcap installed in WinPcap API-compatible mode
- Optional: Npcap SDK for linking (Windows)

## Needed in Windows

- Npcap-sdk stored in some local location like - "C:\\npcap-sdk\\Lib\\x64"
- A .cargo/congif.toml file

    ```toml
        [target.x86_64-pc-windows-msvc]
        rustflags = ["-L", "C:\\npcap-sdk\\Lib\\x64"]
    ```
