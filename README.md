# cord

A native, moddable chat client for **Discord**, **Matrix** and **Stoat** in one app.

Servers, spaces, DMs and friends from all three networks share a single server bar and a single friends list. Every object carries a small network badge, everything else behaves the same.

## Status

Early development. Nothing here is usable yet.

## Goals

- Full Discord feature parity: text, threads, forums, reactions, embeds, stickers, voice, video, screen share with audio, stages, soundboard, events, server management
- Matrix (via `matrix-rust-sdk`, with E2EE) and Stoat on the same footing
- Built-in client mods that usually need Vencord: message logger, hidden channels, silent typing, emoji and sticker passthrough, stream quality unlock, Spotify controls, translation and more, each toggleable
- Block based UI: every panel is a block. Pick a preset on first launch, flip simple toggles, or rearrange everything in edit mode. Layouts are plain files and can be shared
- Themes with hot reload, design tokens and full QML component overrides
- Mod API in QML/JS and WASM, the built-in features use the same API

## Stack

Rust core, Qt 6 / QML UI through [cxx-qt](https://github.com/KDAB/cxx-qt). Voice through PipeWire and Opus, Discord E2EE through libdave, Matrix and Stoat calls through LiveKit, screen capture through the XDG desktop portal with hardware encoding.

## Building

Requires Rust, Qt 6.8+ (base, declarative, multimedia), PipeWire, libsecret.

```sh
cargo run -p cord
```

## Disclaimer

Using third party clients on Discord is against Discord's Terms of Service and may get your account suspended. Use at your own risk. Matrix and Stoat explicitly allow third party clients.

## License

GPL-3.0-or-later
