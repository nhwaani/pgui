# PGUI

A high performance GUI to query & manage PostgreSQL and MySQL databases.

Written in [GPUI](https://gpui.rs) with [GPUI Component](https://github.com/longbridge/gpui-component)

<img src="https://github.com/duanebester/pgui/blob/main/assets/screenshots/pgui-dual.png" height="400px" />

### Supported databases

- **PostgreSQL** (any reasonably recent version)
- **MySQL 8.4 LTS** (the wire protocol and `information_schema` queries are
  also compatible with the 8.0 series)

### Connections

Connections and query history are saved to a SQLite database at
`~/.pgui/pgui.db`. The connection form lets you pick a driver per saved
connection; the default port adjusts automatically (5432 for Postgres,
3306 for MySQL).

Database passwords and SSH key passphrases are stored in the host OS
secure store via the Keyring crate, never in the SQLite database.

### SSH tunnels

Any saved connection can be routed through an SSH tunnel. Toggle
**"Connect through SSH tunnel"** in the connection form and provide:

- SSH host / port / user
- Authentication: **SSH Agent** (uses `SSH_AUTH_SOCK`) or a **Private Key
  File** (with optional passphrase saved to the keyring)

pgui opens a local-port-forward tunnel (`127.0.0.1:<random>` →
`<db host>:<db port>` over SSH) and connects sqlx to the local end. The
tunnel is torn down when you disconnect.

Password-based SSH authentication is intentionally not supported; use a
key or an agent.

### Agent Panel

Only Anthropic support w/ `ANTHROPIC_API_KEY` via enviroment.

### AI Completions (Cmd+.)

AI Completions are triggered via code actions (cmd + .) or via the inline completions toggle.

> Note: currently hard-coded to claude haiku 4.5

### Building

See [Mac App Build](./MAC_APP_BUILD.md) for building locally on MacOS

### Local development databases

`docker-compose.yml` brings up both engines for end-to-end testing:

```bash
docker compose up -d            # both Postgres and MySQL
docker compose up -d mysql      # MySQL only
docker compose up -d db         # Postgres only
```

| Engine     | Host        | Port | DB     | User   | Password |
|------------|-------------|------|--------|--------|----------|
| PostgreSQL | `localhost` | 5432 | `test` | `test` | `test`   |
| MySQL 8.4  | `localhost` | 3306 | `test` | `test` | `test`   |

Both containers seed themselves on first start (`init.sql` and
`init.mysql.sql`). To rebuild from scratch (re-runs the seed scripts):

```bash
docker compose down -v && docker compose up -d
```

### Running the test suite

```bash
cargo test           # 35 unit + integration tests, no DB required
cargo run --bin build-app  # produce PGUI.app
```

Integration tests live under [`tests/`](./tests) and use a process-wide
in-memory keyring backend (see [`tests/common/mod.rs`](./tests/common/mod.rs)).
Nothing in the test suite touches the real macOS Keychain or a live
database.
