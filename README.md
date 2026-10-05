# Runtipi Fleet Dashboard

A small Rust collector that polls trusted Runtipi Companion instances and generates [Homepage](https://gethomepage.dev/) `services.yaml`. It is designed for a private Tailscale network: the collector has no inbound API, needs no Docker socket, and reads one bearer token per server from a mounted file.

The Runtipi app-store and Companion changes are captured as implementation handoffs under `docs/handoffs/`. Companion inventory serving is being implemented separately against the versioned inventory contract in this repository.

## Behaviour

- Polls companions concurrently every 60 seconds by default, with independent timeouts.
- Uses persistent server UUID plus app URN as identity.
- Keeps the last successful inventory when a server is offline.
- Removes an app only following a successful inventory that omits it.
- Keeps stopped apps visible and marks their status in the tile description.
- Supports browser-facing URL overrides independently of discovery URLs.
- Writes Homepage configuration atomically and only when bytes change.
- Strictly rejects unknown inventory fields to keep the companion contract an allowlist.
- Exits on collector-wide filesystem or rendering failures so the container supervisor can report and restart a broken instance.

## Container image

Versioned `linux/amd64` and `linux/arm64` images are published to GHCR:

```sh
docker pull ghcr.io/eckphi/runtipi-fleet-dashboard:0.1.0
```

For reproducible deployments, pin the digest recorded by the release workflow instead of relying on a mutable tag.

The container runs as UID/GID `65532:65532`. `/state` and `/homepage` must be writable by that identity; `/config` and `/run/secrets` should be read-only. A minimal invocation is:

```sh
docker run --rm \
  -v "$PWD/fleet.yaml:/config/fleet.yaml:ro" \
  -v "$PWD/secrets:/run/secrets:ro" \
  -v fleet-state:/state \
  -v homepage-config:/homepage \
  ghcr.io/eckphi/runtipi-fleet-dashboard:0.1.0
```

When using bind mounts, prepare ownership on the host:

```sh
sudo chown -R 65532:65532 state homepage
```

The development Compose stack includes a one-shot, capability-limited initializer for named-volume ownership.

## Run from source

Copy `fleet.example.yaml` to the ignored `fleet.yaml`, create the referenced ignored token files, and run:

```sh
cargo run -- --config fleet.yaml --state-dir state --output homepage/services.yaml --once
```

For the development stack, adjust the Homepage allowed host and then use:

```sh
docker compose -f deploy/compose.yaml up --build
```

Do not commit `fleet.yaml`, `.env` files, state, real hostnames, tokens, or generated Homepage output. See [architecture](docs/architecture.md), the [Runtipi app handoff](docs/handoffs/runtipi-app.md), and the [Companion handoff](docs/handoffs/runtipi-companion.md).

## Release verification

Release images include an SBOM and build provenance. Verify the GitHub attestation with:

```sh
gh attestation verify \
  oci://ghcr.io/eckphi/runtipi-fleet-dashboard:0.1.0 \
  -R EckPhi/runtipi-fleet-dashboard
```

## Compatibility

The handoff targets Runtipi v4.10.2 and dynamic Compose schema 2. It must be installed on a disposable v4.10.2 instance before the Runtipi app package is released. Current Runtipi documentation may not exactly reflect that older target.

## License

MIT
