# Handoff: Runtipi app-store package

Target: the private/custom Runtipi app-store repository. Do not publish or push this draft. Validate on a disposable Runtipi **v4.10.2** instance; current docs describe a moving target.

## Package layout

```text
apps/runtipi-fleet-dashboard/
  config.json
  docker-compose.yml
  metadata/description.md
  metadata/logo.jpg
```

Use dynamic Compose schema 2. `config.json` should set `id` to `runtipi-fleet-dashboard`, `min_tipi_version` to `4.10.2`, `dynamic_config` true, `supported_architectures` to `amd64` and `arm64`, and expose Homepage as the main GUI. Pin both images by immutable digest for release; do not use `latest`.

## Service wiring

- `homepage`: upstream Homepage, main service, internal port 3000. Mount `${APP_DATA_DIR}/homepage:/app/config:ro`. Set `HOMEPAGE_ALLOWED_HOSTS` from an install field, with clear text that it is not authentication.
- `collector`: dashboard image, no ports and not on the Runtipi main network unless outbound connectivity demands it. Mount `${APP_DATA_DIR}/config/fleet.yaml:/config/fleet.yaml:ro`, `${APP_DATA_DIR}/secrets:/run/secrets:ro`, `${APP_DATA_DIR}/state:/state`, and `${APP_DATA_DIR}/homepage:/homepage`.
- `volume-init`: one-shot root service that grants UID/GID `65532:65532` ownership of the collector state and shared Homepage configuration directories, then exits. Retain only `CHOWN`, drop all other capabilities, and require successful completion before starting either service.
- Both: `read_only: true`, `cap_drop: [ALL]`, `security_opt: [no-new-privileges:true]`, `/tmp` tmpfs, conservative memory/CPU limits.
- Never mount `/var/run/docker.sock`. Never mount the secrets or state directory into Homepage.

The collector must be able to write `services.yaml` while Homepage sees the same directory read-only. Seed any other required Homepage configuration during installation or document its creation. Confirm whether Homepage v1.4.6 reloads `services.yaml` in an open browser; if it does not, document the required browser refresh.

## Private runtime setup

Installation instructions should tell the operator to create, outside Git:

1. `${APP_DATA_DIR}/config/fleet.yaml` based on the dashboard example.
2. One mode-`0600` token file per server under `${APP_DATA_DIR}/secrets`.
3. A Tailscale ACL allowing only the Runtipi host to each companion port.
4. A Homepage hostname reachable only through Tailscale, or a separate authentication proxy if exposure is required.

## Validation gate

Run app-store schema validation, install/start/restart/update/backup/restore/uninstall tests on v4.10.2, amd64 smoke tests, and arm64 image inspection. Verify that app data survives update; secrets never appear in generated Compose, logs, or Homepage; Homepage cannot read secrets; the collector has no inbound port; and the app works without a Docker socket.
