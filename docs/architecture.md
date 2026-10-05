# Architecture

The collector reads `fleet.yaml`, loads each token only for its corresponding request, and calls `GET /v1/inventory`. Successful responses are validated, written to one JSON cache file per configured server UUID, and assembled into a deterministic Homepage document. A failed request uses that UUID's prior cache. A successful empty or reduced inventory is authoritative, so removed apps disappear.

The Companion discovery URL and each app's browser URL serve different audiences. Companion URLs normally use MagicDNS or Tailscale addresses reachable from the Runtipi host. App URLs must be reachable from the user's browser; `url_overrides` correct these without changing discovery.

## Trust boundaries

- Companion authentication is a distinct bearer token per host.
- Tailscale policy should allow the dashboard host to reach only the Companion port.
- Companion should bind only to the Tailscale address; it must not listen publicly.
- Homepage mounts only generated configuration. It never mounts tokens, fleet configuration, state, or the Docker socket.
- The collector has no listening port and no Docker socket.
- The collector runs as UID/GID `65532:65532`; only its state and generated-output directories are writable.
- Homepage `HOMEPAGE_ALLOWED_HOSTS` protects host handling; it is not user authentication.

## Cache semantics

Each response must carry the configured persistent server UUID. A mismatch is rejected and cannot overwrite another cache. The cache records both the companion's collection time and the dashboard refresh time. Corrupt cache files stop a cycle instead of being silently discarded; operators should inspect rather than lose the last-known-good state.

## Operations

Logs never include tokens or response bodies. A whole-cycle error leaves the previous generated file untouched. SIGINT stops after the current request set completes. Rolling back the collector is safe because cache and output formats are plain JSON/YAML.
