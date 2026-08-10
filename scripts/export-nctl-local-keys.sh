#!/usr/bin/env bash
# Ops helper: print LOCAL_KEYS_JSON for selected NCTL *users* (already funded).
# Local mode only (SIGN_BACKEND=local). Does NOT include the faucet.
# Faucet is for KMS funding via fund-kms-from-nctl.sh only.
#
# Env:
#   NCTL_CONTAINER  default casper-nctl-2-docker-dev
#   NCTL_USERS      space-separated user ids, default "1 2 3"
set -euo pipefail

NCTL_CONTAINER="${NCTL_CONTAINER:-casper-nctl-2-docker-dev}"
NCTL_USERS="${NCTL_USERS:-1 2 3}"
ASSETS="/app/casper-nctl/assets/net-1"

if ! docker inspect -f '{{.State.Running}}' "$NCTL_CONTAINER" 2>/dev/null | grep -qx true; then
  echo "nctl container not running: $NCTL_CONTAINER" >&2
  exit 1
fi

# shellcheck disable=SC2086
users_csv="$(echo $NCTL_USERS | tr ' ' ',')"

docker exec -e USERS_CSV="$users_csv" "$NCTL_CONTAINER" bash -lc '
set -euo pipefail
python3 - <<PY
import json, os, pathlib
assets = pathlib.Path("'"$ASSETS"'")
out = {}
ids = [int(x) for x in os.environ["USERS_CSV"].split(",") if x.strip()]
if not ids:
    raise SystemExit("NCTL_USERS empty")
for i in ids:
    d = assets / "users" / f"user-{i}"
    if not d.is_dir():
        raise SystemExit(f"missing {d}")
    pk = (d / "public_key_hex").read_text().strip()
    pem = (d / "secret_key.pem").read_text()
    out[pk] = pem
print(json.dumps(out, separators=(",", ":")))
PY
'
