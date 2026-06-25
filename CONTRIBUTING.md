# Contributing to AgentVM

Thanks for your interest in contributing! AgentVM is a community-driven project, and we need all the help we can get.

---

## Table of Contents

- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [How to Contribute](#how-to-contribute)
- [Contribution Areas](#contribution-areas)
- [Code Guidelines](#code-guidelines)
- [Pull Request Process](#pull-request-process)
- [RFC Process](#rfc-process)
- [Community](#community)

---

## Getting Started

### Prerequisites

- **Rust** 1.75+ (for the core, memory, and CLI crates)
- **Node.js** 20+ (for platform adapters and web Studio)
- **Python** 3.11+ (planned for Python SDK)
- **Go** 1.22+ (for the local registry API)
- **Git** 2.30+

### First Time?

1. Star the repo ⭐
2. Read the [specification](spec.md)
3. Check the current implemented surface before choosing work
4. Pick a [`good-first-issue`](https://github.com/agentvm/agentvm/labels/good-first-issue)
5. Ask questions in GitHub Discussions

---

## Development Setup

```bash
# Clone
git clone https://github.com/agentvm/agentvm.git
cd agentvm

# Check the Rust workspace
cargo check --workspace

# Run tests
cargo test --workspace

# Run the full local verification gate
bash scripts/verify.sh

# Run linters
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check

# Smoke the CLI image flow
cargo run -p agentvm-cli -- validate examples/minimal-agent.yaml
cargo run -p agentvm-cli -- init /tmp/agentvm-smoke --template senior-dev --force
cargo run -p agentvm-cli -- pack /tmp/agentvm-smoke --output /tmp/agentvm-smoke.agentvm
cargo run -p agentvm-cli -- unpack /tmp/agentvm-smoke.agentvm --output /tmp/agentvm-restored
# Studio browser bundles use the same unpack command:
# cargo run -p agentvm-cli -- unpack /tmp/agentvm-studio.agentvm.json --output /tmp/agentvm-studio-restored
cargo run -p agentvm-cli -- memory search /tmp/agentvm-smoke tests
cargo run -p agentvm-cli -- checksum examples/minimal-agent.yaml
cargo run -p agentvm-cli -- merge examples/minimal-agent.yaml examples/reviewer-agent.yaml --output /tmp/agentvm-merged.yaml

# Build the web Studio
cd registry/web
npm install
npm run build

# Build platform adapters
cd ../../adapters
npm install
npm test

# Test the local registry API
cd ../registry
go test ./...
```

### Project Structure

```
agentvm/
├── spec.md                 # Specification document
├── AGENTS.md               # Agent instructions for repo automation
├── LICENSE                 # Apache-2.0 license
├── .github/workflows/ci.yml
│
├── crates/                 # Rust crates
│   ├── agentvm-cli/        # CLI binary
│   ├── agentvm-core/       # Core library (image parsing, validation, diff)
│   └── agentvm-memory/     # Local memory loading, recall, consolidation, export
│
├── examples/               # Example Agent Image manifests for smoke tests
│
├── adapters/               # TypeScript platform adapters
│
├── registry/               # Go registry API preview
│   └── web/                # AgentVM Studio React app
```

Planned surfaces in the roadmap include richer runtime execution, deeper exporters/importers, hosted registry auth, SDKs, template marketplace, RFCs, and integration tests. Please do not document them as implemented until the files exist.

---

## How to Contribute

### 1. Find Something to Work On

| Type | Where to Look |
|------|--------------|
| **First contribution** | [`good-first-issue`](https://github.com/agentvm/agentvm/labels/good-first-issue) |
| **Bug fix** | [`bug`](https://github.com/agentvm/agentvm/labels/bug) |
| **New feature** | [`enhancement`](https://github.com/agentvm/agentvm/labels/enhancement) |
| **Documentation** | [`documentation`](https://github.com/agentvm/agentvm/labels/documentation) |
| **Spec change** | Open an RFC first |

### 2. Claim the Issue

- Comment on the issue: "I'd like to work on this"
- Wait for assignment (prevents duplicate work)
- Ask questions if anything is unclear

### 3. Fork & Branch

```bash
# Fork on GitHub, then:
git clone https://github.com/YOUR-USERNAME/agentvm.git
cd agentvm
git remote add upstream https://github.com/agentvm/agentvm.git

# Create a branch
git checkout -b feat/your-feature-name
```

### 4. Make Changes

- Write code
- Write tests
- Update docs if needed
- Run linters

### 5. Submit PR

```bash
git push origin feat/your-feature-name
# Open PR on GitHub
```

---

## Contribution Areas

### 🟢 Easy (Good First Issues)

#### Platform Adapters
Write an adapter that converts an Agent Image to a specific platform's format.

**What you'll learn:** Agent Image format, platform APIs
**Time:** 2-4 hours
**Skills needed:** TypeScript or Python

```typescript
// Example: Adapt to Mistral
export class MistralAdapter implements PlatformAdapter<MistralConfig> {
  readonly platform = "mistral";

  toPlatformFormat(image: AgentImage): MistralConfig {
    return {
      instructions: this.buildInstructions(image),
      knowledge: this.buildKnowledge(image),
    };
  }

  private buildInstructions(image: AgentImage): string {
    // Convert identity + persona to Gemini format
    return `${image.identity.persona}\n\n${image.prompts.system}`;
  }
}
```

#### Agent Templates
Create a pre-built agent template for a common use case.

**What you'll learn:** Agent Image format
**Time:** 1-2 hours
**Skills needed:** YAML, Markdown

```yaml
# templates/customer-support/agent.yaml
apiVersion: agentvm/v1
kind: AgentImage
metadata:
  name: "customer-support"
  version: "1.0.0"
  description: "Template for customer support agents"

identity:
  persona: |
    Friendly, patient customer support agent.
    Always empathetic. Never defensive.
    Escalate when unsure.

skills:
  builtin:
    - id: "ticket-triage"
    - id: "empathetic-responses"
    - id: "escalation-detection"
```

#### Test Cases
Write spec compliance tests for the image validator.

**What you'll learn:** Spec details, testing patterns
**Time:** 1-3 hours
**Skills needed:** Rust or TypeScript

```rust
#[test]
fn test_valid_agent_image() {
    let yaml = r#"
apiVersion: agentvm/v1
kind: AgentImage
metadata:
  name: "test-agent"
  version: "1.0.0"
identity:
  persona: "A helpful assistant"
"#;
    let image = AgentImage::from_yaml(yaml).unwrap();
    assert!(image.validate().is_ok());
}

#[test]
fn test_missing_required_fields() {
    let yaml = r#"
apiVersion: agentvm/v1
kind: AgentImage
metadata:
  name: "test-agent"
  # Missing version!
"#;
    let result = AgentImage::from_yaml(yaml);
    assert!(result.is_err());
}
```

#### Documentation
Write guides, tutorials, or improve existing docs.

**What you'll learn:** The whole project
**Time:** 1-2 hours
**Skills needed:** Markdown, clear writing

---

### 🟡 Medium

#### Runtime Adapters
Implement the runtime interface for a new LLM provider.

**What you'll learn:** Runtime API, LLM APIs
**Time:** 1-2 days
**Skills needed:** TypeScript

See [spec.md § Runtime API](spec.md#4-runtime-api) for the interface.

#### Memory Engine
Implement memory storage, retrieval, or consolidation.

**What you'll learn:** Memory system internals
**Time:** 2-3 days
**Skills needed:** Rust

Key algorithms to implement:
- BM25 for keyword search
- Cosine similarity for semantic search
- Importance-based consolidation
- Recency-weighted retrieval

#### Web UI
Build the React frontend for managing agent images.

**What you'll learn:** Full stack, Agent Image format
**Time:** 1 week
**Skills needed:** React, TypeScript, Tailwind

---

### 🔴 Hard

#### Core Spec Changes
Design and implement changes to the specification itself.

**Process:** Must go through [RFC process](#rfc-process).

#### Registry Server
Build the image registry server.

**What you'll learn:** Go, distributed systems
**Time:** 2-3 weeks
**Skills needed:** Go, PostgreSQL/S3

---

## Code Guidelines

### Rust

```bash
# Format
cargo fmt --all --check

# Lint
cargo clippy --workspace --all-targets -- -D warnings

# Test
cargo test --workspace

# Smoke the CLI
cargo run -p agentvm-cli -- validate examples/minimal-agent.yaml
```

- Use `thiserror` for error types
- Use `serde` for serialization
- Document public APIs with `///` comments
- Write doc tests for examples

### TypeScript

```bash
# Format
npx prettier --write .

# Lint
npx eslint .

# Test
npm test
```

- Use TypeScript strict mode
- Use `zod` for runtime validation
- Write JSDoc for public APIs
- Prefer `async/await` over raw promises

### Go

```bash
# Format
gofmt -w .

# Lint
golangci-lint run

# Test
go test ./...
```

- Follow Effective Go
- Use `context` for cancellation
- Write table-driven tests

### Commit Messages

We use [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add Mistral adapter
fix: memory consolidation crash on empty entries
docs: add quick start guide
test: add image validation tests
refactor: extract memory types into shared crate
chore: update dependencies
```

Scope is optional but encouraged:

```
feat(memory): add BM25 search algorithm
fix(cli): handle missing agent.yaml gracefully
docs(spec): clarify memory consolidation rules
```

---

## Pull Request Process

### Before Submitting

- [ ] Code compiles (`cargo check --workspace`)
- [ ] Tests pass (`cargo test --workspace`)
- [ ] Linters pass (`cargo clippy --workspace --all-targets -- -D warnings`)
- [ ] Formatting passes (`cargo fmt --all --check`)
- [ ] Docs updated if needed
- [ ] Commit messages follow conventions
- [ ] Branch is up to date with main

### PR Template

```markdown
## What

Brief description of the change.

## Why

Why this change is needed.

## How

How the change works.

## Testing

How to test this change.

## Checklist

- [ ] Tests added/updated
- [ ] Docs updated
- [ ] Lint passes
- [ ] No breaking changes (or documented in description)
```

### Review Process

1. **Automated checks** — CI runs tests and linters
2. **Code review** — At least one maintainer reviews
3. **Feedback** — Address review comments
4. **Approval** — Maintainer approves
5. **Merge** — Squash and merge to main

### What Reviewers Look For

- **Correctness** — Does it work? Are edge cases handled?
- **Tests** — Are there tests? Do they cover the important cases?
- **Readability** — Is the code clear? Would you understand it in 6 months?
- **Spec compliance** — Does it follow the spec?
- **Performance** — Any obvious performance issues?
- **Security** — Any security concerns?

---

## RFC Process

Major changes to the spec or architecture go through an RFC (Request for Comments).

### When to Write an RFC

- New spec section
- Breaking change to existing spec
- New major component
- Change to data format
- New protocol

### When NOT to Write an RFC

- Bug fix
- Minor improvement
- New exporter/adapter (just implement it)
- Documentation

### How to Write an RFC

1. Create a file in `rfcs/`:
   ```
   rfcs/0001-memory-consolidation.md
   ```

2. Use this template:
   ```markdown
   # RFC 0001: Memory Consolidation Algorithm

   ## Summary
   One paragraph explanation.

   ## Motivation
   Why is this needed?

   ## Detailed Design
   The actual proposal.

   ## Alternatives Considered
   What else was considered and why it was rejected.

   ## Unresolved Questions
   What still needs to be figured out.

   ## References
   Related work, papers, prior art.
   ```

3. Open a PR with the RFC
4. Community discussion (minimum 2 weeks)
5. Core team vote (requires 2/3 majority)
6. If approved, implement

---

## Community

### Getting Help

- **GitHub Discussions:** Ask questions, share ideas

### Code of Conduct

Be kind. Be respectful. Be constructive. We're all here to build something cool.

Specifically:
- **Do:** Be welcoming, patient, and constructive
- **Do:** Give and receive feedback gracefully
- **Do:** Focus on what's best for the community
- **Don't:** Be a jerk
- **Don't:** Harass anyone
- **Don't:** Spam or self-promote excessively

### Recognition

Significant contributors may be recognized in release notes and invited to help maintain the project over time.

---

## Questions?

Open a [GitHub Discussion](https://github.com/agentvm/agentvm/discussions). We're happy to help!

---

*Thank you for making AgentVM better! 🎉*
