# Handoff: Runtipi Companion inventory service

Target repository: `EckPhi/runtipi-companion`. Do not publish this handoff or push changes from the dashboard worktree.

## Deliverable

Add an optional long-running `serve` command without changing existing backup/update behaviour:

```text
runtipi-companion serve \
  --listen ${TAILSCALE_IP}:8765 \
  --token-file /etc/runtipi-companion/dashboard.token \
  --server-id-file /var/lib/runtipi-companion/server-id
```

The server ID file is created once with a UUIDv4 using mode `0600`, then reused across hostname/display-name changes. Refuse wildcard binds by default; require an explicit unsafe flag if maintainers decide to support them. Read the token file at request time or support a reload signal so rotation does not require rebuilding.

## HTTP contract

- `GET /v1/health`: bearer authentication required; return `{"status":"ok","protocol_version":"1"}`.
- `GET /v1/inventory`: bearer authentication required; return the exact shape in `../contracts/inventory-v1.example.json`.
- Return `401` with no authentication detail for missing/incorrect tokens. Compare tokens in constant time.
- Set JSON content type, disable CORS, cap request header size, and log no Authorization value.

Allowed app fields are only `urn`, `name`, `status`, `browser_url`, and `icon`. Never serialize raw Runtipi configuration, environment variables, form values, credentials, volume paths, backup destinations, or logs. Normalize status to `running`, `stopped`, `installing`, `updating`, `error`, or `unknown`.

The URN is Runtipi's full `<app-id>:<store-slug>` identifier. `browser_url` is the URL a user's browser can open, not a container-only address. `icon` is an optional Homepage-compatible icon string, not arbitrary HTML.

## Adapter

Introduce an `InventoryProvider` interface so the HTTP layer is independent of discovery. Implement and fixture-test a Runtipi v4.10.2 adapter. First try the existing authenticated local API/session facilities already used by Companion; if this proves fragile, use read-only file discovery behind the same interface. In either implementation, map into the explicit DTO before serialization.

## systemd unit

Ship `runtipi-companion-inventory.service` with `User`/`Group` dedicated to Companion where practical, `Restart=on-failure`, `NoNewPrivileges=true`, `PrivateTmp=true`, `ProtectSystem=strict`, `ProtectHome=true`, explicit `ReadWritePaths` only for the ID file directory, and read access only to the chosen Runtipi discovery source. Pass paths through an environment file; never place a bearer token directly in the unit.

## Acceptance tests

1. Stable UUID survives restart and display-name change.
2. Missing/wrong bearer tokens receive 401; correct per-server token succeeds.
3. Raw app secrets and unrelated Runtipi fields cannot appear in JSON.
4. Duplicate app IDs across two servers remain valid because UUID differs.
5. Stopped apps remain present and use `stopped`.
6. Adapter fixtures match v4.10.2 and malformed source data fails closed.
7. Service is reachable on the Tailscale address and unreachable on public interfaces.
