#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/agentvm-verify.XXXXXX")"
REGISTRY_PID=""

cleanup() {
  if [[ -n "${REGISTRY_PID}" ]]; then
    kill "${REGISTRY_PID}" >/dev/null 2>&1 || true
    wait "${REGISTRY_PID}" >/dev/null 2>&1 || true
  fi
  rm -rf "${TMP_DIR}"
}
trap cleanup EXIT

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "missing required command: $1" >&2
    exit 127
  fi
}

run() {
  echo "==> $*"
  "$@"
}

pick_port() {
  python3 - <<'PY'
import socket

with socket.socket() as sock:
    sock.bind(("127.0.0.1", 0))
    print(sock.getsockname()[1])
PY
}

require_cmd cargo
require_cmd go
require_cmd npm
require_cmd python3

cd "${ROOT}"

run cargo fmt --all --check
run cargo test --workspace
run cargo clippy --workspace --all-targets -- -D warnings

IMAGE_DIR="${TMP_DIR}/agentvm-smoke"
SECRET_SCAN_DIR="${TMP_DIR}/agentvm-secret-scan"
RESTORED_DIR="${TMP_DIR}/agentvm-restored"
MERGED_YAML="${TMP_DIR}/agentvm-merged.yaml"
OPENCLAW_EXPORT="${TMP_DIR}/agentvm-openclaw-export"
OPENCLAW_IMPORT="${TMP_DIR}/agentvm-openclaw-import"
CHATGPT_EXPORT="${TMP_DIR}/agentvm-chatgpt-export"
CHATGPT_IMPORT="${TMP_DIR}/agentvm-chatgpt-import"
CLAUDE_EXPORT="${TMP_DIR}/agentvm-claude-export"
CLAUDE_IMPORT="${TMP_DIR}/agentvm-claude-import"
GEMINI_EXPORT="${TMP_DIR}/agentvm-gemini-export"
GEMINI_IMPORT="${TMP_DIR}/agentvm-gemini-import"
OLLAMA_EXPORT="${TMP_DIR}/agentvm-ollama-export"
OLLAMA_IMPORT="${TMP_DIR}/agentvm-ollama-import"
REGISTRY_PULL="${TMP_DIR}/agentvm-registry-pull.json"
REGISTRY_RESTORED="${TMP_DIR}/agentvm-registry-restored"
REGISTRY_DATA="${TMP_DIR}/agentvm-registry.json"
REGISTRY_ADDR="127.0.0.1:$(pick_port)"

run cargo run -p agentvm-cli -- validate examples/minimal-agent.yaml
run cargo run -p agentvm-cli -- init "${IMAGE_DIR}" --template senior-dev --force
run cargo run -p agentvm-cli -- security scan "${IMAGE_DIR}" --strict
run cargo run -p agentvm-cli -- init "${SECRET_SCAN_DIR}" --template senior-dev --force
printf '# Episodic Memory\n\napi_key: sk-test-secret-token-value\n' > "${SECRET_SCAN_DIR}/memory/episodic.md"
if cargo run -p agentvm-cli -- security scan "${SECRET_SCAN_DIR}" --strict; then
  echo "expected strict security scan to fail for secret-like fixture" >&2
  exit 1
fi
run cargo run -p agentvm-cli -- pack "${IMAGE_DIR}" --output "${TMP_DIR}/agentvm-smoke.agentvm"
run cargo run -p agentvm-cli -- unpack "${TMP_DIR}/agentvm-smoke.agentvm" --output "${RESTORED_DIR}"
run cargo run -p agentvm-cli -- memory search "${IMAGE_DIR}" tests
run cargo run -p agentvm-cli -- run "${IMAGE_DIR}" --dry-run --platform ollama --prompt "Summarize memory"
run cargo run -p agentvm-cli -- checksum examples/minimal-agent.yaml
run cargo run -p agentvm-cli -- merge examples/minimal-agent.yaml examples/reviewer-agent.yaml --output "${MERGED_YAML}"
run cargo run -p agentvm-cli -- validate "${MERGED_YAML}"
run cargo run -p agentvm-cli -- export examples/reviewer-agent.yaml --to openclaw --output "${OPENCLAW_EXPORT}"
run cargo run -p agentvm-cli -- import openclaw --workspace "${OPENCLAW_EXPORT}" --output "${OPENCLAW_IMPORT}" --force
run cargo run -p agentvm-cli -- validate "${OPENCLAW_IMPORT}/agent.yaml"
run cargo run -p agentvm-cli -- export examples/reviewer-agent.yaml --to chatgpt --output "${CHATGPT_EXPORT}"
run cargo run -p agentvm-cli -- import chatgpt --workspace "${CHATGPT_EXPORT}" --output "${CHATGPT_IMPORT}" --force
run cargo run -p agentvm-cli -- validate "${CHATGPT_IMPORT}/agent.yaml"
run cargo run -p agentvm-cli -- export examples/reviewer-agent.yaml --to claude --output "${CLAUDE_EXPORT}"
run cargo run -p agentvm-cli -- import claude --workspace "${CLAUDE_EXPORT}" --output "${CLAUDE_IMPORT}" --force
run cargo run -p agentvm-cli -- validate "${CLAUDE_IMPORT}/agent.yaml"
run cargo run -p agentvm-cli -- export examples/reviewer-agent.yaml --to gemini --output "${GEMINI_EXPORT}"
run cargo run -p agentvm-cli -- import gemini --workspace "${GEMINI_EXPORT}" --output "${GEMINI_IMPORT}" --force
run cargo run -p agentvm-cli -- validate "${GEMINI_IMPORT}/agent.yaml"
run cargo run -p agentvm-cli -- export examples/reviewer-agent.yaml --to ollama --output "${OLLAMA_EXPORT}"
run cargo run -p agentvm-cli -- import ollama --workspace "${OLLAMA_EXPORT}" --output "${OLLAMA_IMPORT}" --force
run cargo run -p agentvm-cli -- validate "${OLLAMA_IMPORT}/agent.yaml"
run cargo run -p agentvm-cli -- skills list "${IMAGE_DIR}"
run cargo run -p agentvm-cli -- skills add "${IMAGE_DIR}" registry://skills/github-advanced --version 3.1.0
run cargo run -p agentvm-cli -- skills remove "${IMAGE_DIR}" github-advanced
run cargo run -p agentvm-cli -- version bump "${IMAGE_DIR}" patch --message "Smoke version bump"
run cargo run -p agentvm-cli -- changelog "${IMAGE_DIR}"
run cargo run -p agentvm-cli -- validate "${IMAGE_DIR}/agent.yaml"

(
  cd "${ROOT}/registry"
  AGENTVM_REGISTRY_ADDR="${REGISTRY_ADDR}" AGENTVM_REGISTRY_DATA="${REGISTRY_DATA}" go run .
) &
REGISTRY_PID=$!

python3 - <<PY
import json
import time
import urllib.error
import urllib.request

base = "http://${REGISTRY_ADDR}"

for _ in range(50):
    try:
        with urllib.request.urlopen(base + "/v1/health", timeout=0.2) as res:
            if res.status == 200:
                break
    except OSError:
        time.sleep(0.1)
else:
    raise SystemExit("registry did not become ready")

def options(origin):
    req = urllib.request.Request(
        base + "/v1/images",
        method="OPTIONS",
        headers={"Origin": origin, "Access-Control-Request-Method": "POST"},
    )
    with urllib.request.urlopen(req) as res:
        return res.status, res.headers.get("Access-Control-Allow-Origin")

assert options("http://127.0.0.1:5173") == (204, "http://127.0.0.1:5173")
assert options("https://example.com") == (204, None)

bad = json.dumps({
    "owner": "ci",
    "name": "bad",
    "version": "1.0.0",
    "files": {"../escape.txt": "nope"},
}).encode()
req = urllib.request.Request(
    base + "/v1/images",
    data=bad,
    method="POST",
    headers={"Content-Type": "application/json"},
)
try:
    urllib.request.urlopen(req)
except urllib.error.HTTPError as err:
    assert err.code == 400
else:
    raise AssertionError("unsafe registry payload was accepted")
PY

run cargo run -p agentvm-cli -- registry push "${IMAGE_DIR}" --owner local --url "http://${REGISTRY_ADDR}"
run cargo run -p agentvm-cli -- registry list --url "http://${REGISTRY_ADDR}"
run cargo run -p agentvm-cli -- registry search senior --url "http://${REGISTRY_ADDR}"
run cargo run -p agentvm-cli -- registry pull local/senior-dev:1.0.1 --output "${REGISTRY_PULL}" --url "http://${REGISTRY_ADDR}"
test -s "${REGISTRY_PULL}"
run cargo run -p agentvm-cli -- unpack "${REGISTRY_PULL}" --output "${REGISTRY_RESTORED}"
run cargo run -p agentvm-cli -- validate "${REGISTRY_RESTORED}/agent.yaml"

(
  cd "${ROOT}/registry"
  run go test ./...
  run go test ./storage -run TestFileStore
)

(
  cd "${ROOT}/registry/web"
  if [[ ! -d node_modules ]]; then
    run npm ci
  fi
  run npm run build
  run npm run smoke
)

(
  cd "${ROOT}/adapters"
  if [[ ! -d node_modules ]]; then
    run npm ci
  fi
  run npm test
)

echo "AgentVM verification passed."
