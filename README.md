# Runtipi Fleet Dashboard

A small Rust collector that polls trusted Runtipi Companion instances and generates [Homepage](https://gethomepage.dev/) `services.yaml`. It is designed for a private Tailscale network: the collector has no inbound API, needs no Docker socket, and reads one bearer token per server from a mounted file.

This repository is local and unpublished. The Runtipi app-store and Companion changes are intentionally captured as handoffs under `docs/handoffs/`, not published or applied to those repositories.

## Behaviour

- Polls companions concurrently every 60 seconds by default, with independent timeouts.
- Uses persistent server UUID plus app URN as identity.
- Keeps the last successful inventory when a server is offline.
- Removes an app only following a successful inventory that omits it.
- Keeps stopped apps visible and marks their status in the tile description.
- Supports browser-facing URL overrides independently of discovery URLs.
- Writes Homepage configuration atomically and only when bytes change.
- Strictly rejects unknown inventory fields to keep the companion contract an allowlist.

## Run locally

Copy `fleet.example.yaml` to the ignored `fleet.yaml`, create the referenced ignored token files, and run:

```sh
cargo run -- --config fleet.yaml --state-dir state --output homepage/services.yaml --once
```

For the development stack, adjust the Homepage allowed host and then use:

```sh
docker compose -f deploy/compose.yaml up --build
```

Do not commit `fleet.yaml`, `.env` files, state, real hostnames, tokens, or generated Homepage output. See [architecture](docs/architecture.md), the [Runtipi app handoff](docs/handoffs/runtipi-app.md), and the [Companion handoff](docs/handoffs/runtipi-companion.md).

## Compatibility

The handoff targets Runtipi v4.10.2 and dynamic Compose schema 2. It must be installed on a disposable v4.10.2 instance before release. Current Runtipi documentation may not exactly reflect that older target.

## License

MIT
