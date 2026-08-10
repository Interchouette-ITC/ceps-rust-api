#!/usr/bin/env bash
# Ops helper only: print LOCAL_KEYS_JSON for NCTL faucet + users (already funded).
# For SIGN_BACKEND=local tests. Not KMS (see fund-kms-from-nctl.sh).
# Not an e2e runner.
set -euo pipefail

NCTL_CONTAINER="${NCTL_CONTAINER:-casper-nctl-2-docker-dev}"
ASSETS="/app/casper-nctl/assets/net-1"

if ! docker inspect -f '{{.State.Running}}' "$NCTL_CONTAINER" 2>/dev/null | grep -qx true; then
  echo "nctl container not running: $NCTL_CONTAINER" >&2
  exit 1
fi

# Build { "public_key_hex": "<pem>", ... } for faucet + user-1..N
docker exec "$NCTL_CONTAINER" bash -lc "
set -euo pipefail
ASSETS='$ASSETS'
python3 - <<'PY'
import json, pathlib
assets = pathlib.Path('$ASSETS')
out = {}

def add(dirpath):
    pk = (dirpath / 'public_key_hex').read_text().strip()
    pem = (dirpath / 'secret_key.pem').read_text()
    out[pk] = pem

add(assets / 'faucet')
users = sorted((assets / 'users').glob('user-*'), key=lambda p: int(p.name.split('-')[1]))
for u in users:
    add(u)
print(json.dumps(out, separators=(',', ':')))
PY
"
