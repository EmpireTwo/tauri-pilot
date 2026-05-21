---
title: Plugin Setup
description: How to add tauri-plugin-pilot to your Tauri application for interactive testing.
---

This guide walks you through integrating `tauri-plugin-pilot` into an existing Tauri v2 application.

## 1. Add the dependency

In your app's `src-tauri/Cargo.toml`, add the plugin under `[dependencies]`:

```toml
# src-tauri/Cargo.toml
[dependencies]
tauri-plugin-pilot = { git = "https://github.com/mpiton/tauri-pilot" }
```

## 2. Register the plugin

Register the plugin in your app entry point using a `#[cfg(debug_assertions)]` guard:

```rust
// src-tauri/src/main.rs
fn main() {
    let mut builder = tauri::Builder::default();

    #[cfg(debug_assertions)]
    {
        builder = builder.plugin(tauri_plugin_pilot::init());
    }

    builder.run(tauri::generate_context!()).expect("error running app");
}
```

## 3. Debug-only compilation

The `#[cfg(debug_assertions)]` guard is intentional and important:

- The plugin is **only compiled and included in debug builds**
- Production builds (`cargo build --release`) will not include the plugin
- There is zero runtime overhead or binary size impact in production
- No need to strip or disable the plugin before shipping

## 4. Socket path

Once your app starts in dev mode, the plugin creates a Unix socket at:

```text
/tmp/tauri-pilot-{identifier}.sock
```

The `{identifier}` value comes from the `identifier` field in your `tauri.conf.json`.

**Example:** an app with identifier `com.myapp.dev` creates the socket at:

```text
/tmp/tauri-pilot-com.myapp.dev.sock
```

The CLI auto-discovers this socket when you run commands.

### Alternative: loopback TCP transport

In sandboxed environments where the default Unix socket / Named Pipe
transport is unavailable, build the plugin with the `tcp-transport` feature:

```toml
# src-tauri/Cargo.toml
[dependencies]
tauri-plugin-pilot = { git = "https://github.com/mpiton/tauri-pilot", features = ["tcp-transport"] }
```

The plugin then binds `127.0.0.1:<random port>` instead and writes the chosen
port to a discovery file at:

```text
${TMPDIR}/tauri-pilot/{identifier}/pilot.port
```

The file is created with permissions `0o600` and its parent directory with
`0o700`. The CLI picks up the newest `pilot.port` under `${TMPDIR}/tauri-pilot/`
automatically; pass `--addr 127.0.0.1:<port>` (or set `TAURI_PILOT_ADDR`) to
override. The accept loop rejects any peer whose IP is not loopback, so the
listener is reachable only from the same host. The `tcp-transport` feature is
mutually exclusive with the Unix socket / Named Pipe transports — pick one.

## 5. Permissions

The plugin requires the `pilot:default` permission for its internal `__callback` IPC command. Add it to your capability file (e.g. `src-tauri/capabilities/default.json`):

```json
{
  "permissions": ["core:default", "pilot:default"]
}
```

Without this permission, eval commands will time out with "eval timed out after 10s".

## 6. Verify the setup

Start your Tauri app in development mode, then test the connection from a second terminal:

```bash
# Terminal 1 — start your app
cargo tauri dev

# Terminal 2 — verify the plugin is reachable
tauri-pilot ping
# Expected output: pong
```

If you see `pong`, the plugin is running and the CLI can reach it. You are ready to start using the snapshot/action workflow.
