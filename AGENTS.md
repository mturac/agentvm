# AgentVM Agent Instructions

You are working in an open-source Rust workspace.

## Priorities

1. Keep changes small and shippable.
2. Read the real files before editing.
3. Preserve public API and spec compatibility unless the task explicitly asks for a breaking change.
4. Verify with the smallest relevant command.
5. Report exact check results.

## Current Project Shape

AgentVM is currently Rust core/memory crates, CLI, TypeScript platform adapters, Go registry API, browser Studio with Safety Scan blocking for package/publish flows, and specification documents. Do not claim SDKs or extra crates exist unless they are present in the repository.

The current implemented crates are:

- `crates/agentvm-core`
- `crates/agentvm-cli`
- `crates/agentvm-memory`

## Commands

Use the repository's actual commands:

```bash
bash scripts/verify.sh
cargo fmt --all --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo run -p agentvm-cli -- validate examples/minimal-agent.yaml
cargo run -p agentvm-cli -- init /tmp/agentvm-smoke --template senior-dev --force
cargo run -p agentvm-cli -- security scan /tmp/agentvm-smoke --strict
cargo run -p agentvm-cli -- pack /tmp/agentvm-smoke --output /tmp/agentvm-smoke.agentvm
cargo run -p agentvm-cli -- unpack /tmp/agentvm-smoke.agentvm --output /tmp/agentvm-restored
cargo run -p agentvm-cli -- memory search /tmp/agentvm-smoke tests
cargo run -p agentvm-cli -- run /tmp/agentvm-smoke --dry-run --platform ollama --prompt "Summarize memory"
cargo run -p agentvm-cli -- checksum examples/minimal-agent.yaml
cargo run -p agentvm-cli -- merge examples/minimal-agent.yaml examples/reviewer-agent.yaml --output /tmp/agentvm-merged.yaml
cargo run -p agentvm-cli -- export examples/reviewer-agent.yaml --to openclaw --output /tmp/agentvm-openclaw-export
cargo run -p agentvm-cli -- import openclaw --workspace /tmp/agentvm-openclaw-export --output /tmp/agentvm-openclaw-import --force
cargo run -p agentvm-cli -- validate /tmp/agentvm-openclaw-import/agent.yaml
cargo run -p agentvm-cli -- export examples/reviewer-agent.yaml --to chatgpt --output /tmp/agentvm-chatgpt-export
cargo run -p agentvm-cli -- import chatgpt --workspace /tmp/agentvm-chatgpt-export --output /tmp/agentvm-chatgpt-import --force
cargo run -p agentvm-cli -- validate /tmp/agentvm-chatgpt-import/agent.yaml
cargo run -p agentvm-cli -- export examples/reviewer-agent.yaml --to claude --output /tmp/agentvm-claude-export
cargo run -p agentvm-cli -- import claude --workspace /tmp/agentvm-claude-export --output /tmp/agentvm-claude-import --force
cargo run -p agentvm-cli -- validate /tmp/agentvm-claude-import/agent.yaml
cargo run -p agentvm-cli -- export examples/reviewer-agent.yaml --to gemini --output /tmp/agentvm-gemini-export
cargo run -p agentvm-cli -- import gemini --workspace /tmp/agentvm-gemini-export --output /tmp/agentvm-gemini-import --force
cargo run -p agentvm-cli -- validate /tmp/agentvm-gemini-import/agent.yaml
cargo run -p agentvm-cli -- export examples/reviewer-agent.yaml --to ollama --output /tmp/agentvm-ollama-export
cargo run -p agentvm-cli -- import ollama --workspace /tmp/agentvm-ollama-export --output /tmp/agentvm-ollama-import --force
cargo run -p agentvm-cli -- validate /tmp/agentvm-ollama-import/agent.yaml
cargo run -p agentvm-cli -- skills list /tmp/agentvm-smoke
cargo run -p agentvm-cli -- skills add /tmp/agentvm-smoke registry://skills/github-advanced --version 3.1.0
cargo run -p agentvm-cli -- skills remove /tmp/agentvm-smoke github-advanced
cargo run -p agentvm-cli -- version bump /tmp/agentvm-smoke patch --message "Smoke version bump"
cargo run -p agentvm-cli -- changelog /tmp/agentvm-smoke
cargo run -p agentvm-cli -- registry push /tmp/agentvm-smoke --owner local --url http://127.0.0.1:8787
cargo run -p agentvm-cli -- registry search senior --url http://127.0.0.1:8787
cargo run -p agentvm-cli -- registry pull local/senior-dev:1.0.1 --output /tmp/agentvm-registry-pull.json --url http://127.0.0.1:8787
(cd adapters && npm test)
(cd registry && go test ./...)
(cd registry && go test ./storage -run TestFileStore)
(cd registry/web && npm run build)
(cd registry/web && npm run smoke)
```

For quick iteration:

```bash
cargo check --workspace
```

## Coding Rules

- Prefer clear Rust types and explicit validation.
- Keep YAML/JSON compatibility stable with the AgentVM spec.
- Use `serde` attributes for wire-format naming instead of Rust camelCase fields.
- Add tests for parser, validator, diff, and memory behavior changes.
- Do not add dependencies unless the change needs them.
- Do not add placeholder modules, empty crates, or fake integration surfaces.

## Open Source Readiness

- Keep README and CONTRIBUTING aligned with files that actually exist.
- Do not reference unavailable commands as working.
- Keep license, CI, and ignore files present and accurate.
- Avoid personal machine paths or private workflow assumptions in repo docs.

## Safety

- Never commit secrets, tokens, cookies, service accounts, private keys, or dumps.
- Do not weaken tests, linting, validation, or security checks to make CI pass.
- Do not stage, commit, or push unless explicitly asked.
