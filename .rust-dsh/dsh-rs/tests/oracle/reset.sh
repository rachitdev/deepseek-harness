#!/bin/bash
# Reset both oracle hosts to the same reminder store and write the environment
# diff.mjs needs to /tmp/schedule-oracle.env.
#
#   tests/oracle/reset.sh && . /tmp/schedule-oracle.env \
#     && node tests/oracle/diff.mjs tests/oracle/cases/schedule.mjs
#
# The TS host keeps its sessions in memory, so it is started once to mint two
# session ids, stopped, given a store seeded for those ids, and started again.
# The RS host is seeded with the same store for its own two sessions.
set -euo pipefail
here="$(cd "$(dirname "$0")/../.." && pwd)"
ts_checkout="${TS_CHECKOUT:-$HOME/code/deepseek-harness}"
export PATH="$HOME/.cargo/bin:$PATH"
log() { echo "$@" >&2; }

pkill -f "bin.ts web --port 3090" 2>/dev/null || true
pkill -f "dsh serve --profile /tmp/rs-oracle" 2>/dev/null || true
sleep 2
rm -f /tmp/ts-oracle-home/storages/schedule.json /tmp/rs-oracle-home/storages/schedule.json /tmp/ts-oracle-home/storages/workspace.json
mkdir -p /tmp/ts-oracle-home /tmp/rs-oracle-home/storages /tmp/rs-oracle/sessions

start_ts() {
  (cd "$ts_checkout" && DSH_HOME=/tmp/ts-oracle-home nohup node --import tsx/esm apps/cli/src/bin.ts web --port 3090 --no-open > /tmp/ts-oracle.log 2>&1 < /dev/null &)
  for _ in $(seq 1 60); do grep -q 'token=' /tmp/ts-oracle.log 2>/dev/null && break; sleep 1; done
  grep -o 'token=[A-Za-z0-9_-]*' /tmp/ts-oracle.log | head -1 | cut -d= -f2 > /tmp/ts-token.txt
  sleep 2
}
start_ts
bash /tmp/mk_sessions.sh > /tmp/ts-sessions.txt
S1=$(sed -n 1p /tmp/ts-sessions.txt); S2=$(sed -n 2p /tmp/ts-sessions.txt)
pkill -f "bin.ts web --port 3090"; sleep 2
(cd "$ts_checkout" && node --import tsx/esm /tmp/seed_store.mjs /tmp/ts-oracle-home/storages/schedule.json "$S1" "$S2" >/dev/null)
start_ts

# RS: same data. Its sessions are numbered from 1 in creation order, so the
# store is seeded for session-1 and session-2 and the sessions are created after the host is up.
(cd "$ts_checkout" && node --import tsx/esm /tmp/seed_store.mjs /tmp/rs-oracle-home/storages/schedule.json session-1 session-2 >/dev/null)
rm -rf /tmp/rs-oracle/sessions; mkdir -p /tmp/rs-oracle/sessions
# The oracle converts documents with the same converter the TypeScript host uses:
# the LibreOffice kit's command-line interface, from the checkout under test.
kit="$(ls -d "$ts_checkout"/node_modules/.pnpm/@deepseek-ai+libreoffice-kit@*/node_modules/@deepseek-ai/libreoffice-kit 2>/dev/null | head -1)/lib/cli.js"
python3 - "$kit" <<'PY'
import json, sys
kit = sys.argv[1]
profile = json.load(open("/tmp/rs-oracle/profile.json"))
profile["plugins"]["gui"]["officeConverter"] = {"command": ["node", kit], "timeoutMs": 120000}
# The TypeScript Web host runs with the workspace-write preset, so the oracle
# does too: a blank session logs the same three setup rows.
profile["plugins"]["permission"] = {"sandbox": "workspace-write", "approval": "ask"}
json.dump(profile, open("/tmp/rs-oracle/profile.json", "w"), indent=1)
PY
(cd "$here" && DSH_HOME=/tmp/rs-oracle-home nohup ./target/debug/dsh serve --profile /tmp/rs-oracle/profile.json --store /tmp/rs-oracle/sessions --store-format v4 > /tmp/rs-oracle.log 2>&1 < /dev/null &)
sleep 4
mk_rs() { curl -s -m 10 -X POST -H "authorization: Bearer oracle-token-0123456789abcdef" -H 'content-type: application/json' -d '{"type":"client-request","rpcId":"1","method":"session/create","payload":{"args":{"request":{"cwd":"/tmp"}}}}' http://127.0.0.1:3081/api/session/create | python3 -c "import sys,json;print(json.load(sys.stdin)['result']['value']['sessionId'])"; }
R1=$(mk_rs); R2=$(mk_rs)
TS_TOKEN=$(cat /tmp/ts-token.txt)
log "TS sessions: $S1 $S2   RS sessions: $R1 $R2"
echo "export TS_URL=http://127.0.0.1:3090 TS_TOKEN=$TS_TOKEN RS_URL=http://127.0.0.1:3081 RS_TOKEN=oracle-token-0123456789abcdef TS_S1=$S1 TS_S2=$S2 RS_S1=$R1 RS_S2=$R2" > /tmp/schedule-oracle.env
