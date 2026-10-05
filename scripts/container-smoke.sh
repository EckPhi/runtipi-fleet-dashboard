#!/bin/sh
set -eu

image=${1:?usage: container-smoke.sh IMAGE}
work=$(mktemp -d)
server_pid=
cleanup() {
  if [ -n "$server_pid" ]; then kill "$server_pid" 2>/dev/null || true; fi
  rm -rf "$work" 2>/dev/null || true
}
trap cleanup EXIT INT TERM

mkdir -p "$work/config" "$work/secrets" "$work/state" "$work/homepage"
chmod 0777 "$work/state" "$work/homepage"
printf '%s\n' 'synthetic-token' > "$work/secrets/server.token"

cat > "$work/config/fleet.yaml" <<'YAML'
poll_interval_seconds: 60
request_timeout_seconds: 5
servers:
  - id: "11111111-1111-4111-8111-111111111111"
    name: "Synthetic Server"
    url: "http://127.0.0.1:18765/"
    token_file: "/run/secrets/server.token"
YAML

python3 - <<'PY' &
from http.server import BaseHTTPRequestHandler, HTTPServer
import json

class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        if self.path != "/v1/inventory" or self.headers.get("Authorization") != "Bearer synthetic-token":
            self.send_response(401)
            self.end_headers()
            return
        body = json.dumps({
            "protocol_version": "1",
            "server": {"id": "11111111-1111-4111-8111-111111111111", "name": "Synthetic Server"},
            "collected_at": "2026-01-01T12:00:00Z",
            "apps": [{
                "urn": "notes:synthetic",
                "name": "Synthetic Notes",
                "status": "running",
                "browser_url": "https://notes.example.test",
                "icon": "mdi-note"
            }]
        }).encode()
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, *_):
        pass

HTTPServer(("127.0.0.1", 18765), Handler).serve_forever()
PY
server_pid=$!
sleep 1

docker run --rm --network host \
  -v "$work/config:/config:ro" \
  -v "$work/secrets:/run/secrets:ro" \
  -v "$work/state:/state" \
  -v "$work/homepage:/homepage" \
  "$image" --once

test "$(docker image inspect "$image" --format '{{.Config.User}}')" = "65532:65532"
test -f "$work/state/inventories/11111111-1111-4111-8111-111111111111.json"
grep -q "Synthetic Notes" "$work/homepage/services.yaml"

if docker run --rm --network host \
  -v "$work/config:/config:ro" \
  -v "$work/secrets:/run/secrets:ro" \
  -v "$work/state:/state:ro" \
  -v "$work/homepage:/homepage" \
  "$image" --once; then
  echo "collector unexpectedly succeeded with read-only state" >&2
  exit 1
fi
