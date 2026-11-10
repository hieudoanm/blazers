# Installation

Blazers ships as a single static binary. There is no runtime dependency, no
database server to provision, and no build step required on the target machine.

## Prebuilt binaries

Prebuilt binaries for Linux (x86_64), macOS (Apple Silicon), and Windows
(x86_64) are attached to each
[GitHub Release](https://github.com/hieudoanm/blazers/releases).

```sh
# macOS / Linux
curl -L -o blazers https://github.com/hieudoanm/blazers/releases/latest/download/blazers-<target>
chmod +x blazers
./blazers serve
```

## Build from source

Requires a recent stable Rust toolchain (edition 2024).

```sh
cargo build --release -p blazers
./target/release/blazers serve
```

The workspace is split into four crates:

| Crate            | Purpose                                           |
| ---------------- | ------------------------------------------------- |
| `blazers-core`   | SQLite schema, migrations, models, wire types     |
| `blazers-server` | HTTP API, realtime, admin dashboard, all features |
| `blazers-mcp`    | MCP stdio server exposing Blazers as agent tools  |
| `blazers`        | Thin binary: `serve` (HTTP) and `mcp serve`       |

## Commands

```text
blazers serve        # start the HTTP server + admin dashboard (default)
blazers mcp serve    # start the MCP stdio server
blazers --help       # usage
```

## Configuration

All configuration is via environment variables. Everything has a working
default, so `blazers serve` is enough to get started.

| Variable       | Default                           | Purpose                                    |
| -------------- | --------------------------------- | ------------------------------------------ |
| `PORT`         | `8080`                            | HTTP listen port                           |
| `BLAZERS_DATA` | `~/.blazers`                      | Data directory (SQLite DB, uploaded files) |
| `JWT_SECRET`   | `dev-secret-change-in-production` | JWT signing secret — **set in production** |

The secrets-vault encryption key (AES-256-GCM) is generated automatically on
first run and stored inside `BLAZERS_DATA`. Back up that directory to keep
your secrets decryptable.

## Verify the install

```sh
curl http://localhost:8080/api/health
# {"status":"ok"}
```

Then open <http://localhost:8080/> for the admin dashboard.
