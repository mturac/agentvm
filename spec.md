# AgentVM — Portable Agent Runtime Specification

> **Draft v0.1.0** · 2026-06-25
> "Your agent's brain, packaged and portable."

---

## Table of Contents

1. [Why This Exists](#1-why-this-exists)
2. [Design Principles](#2-design-principles)
3. [Agent Image Specification](#3-agent-image-specification)
4. [Runtime API](#4-runtime-api)
5. [Tool Protocol](#5-tool-protocol)
6. [Memory System](#6-memory-system)
7. [Platform Exporters](#7-platform-exporters)
8. [CLI Reference](#8-cli-reference)
9. [Registry Protocol](#9-registry-protocol)
10. [Architecture](#10-architecture)
11. [Security Model](#11-security-model)
12. [Roadmap](#12-roadmap)
13. [Contributing](#13-contributing)
14. [FAQ](#14-faq)

---

## 1. Why This Exists

### The Problem

200M+ people use AI agents daily. Every one of those agents is locked to its platform:

```
ChatGPT user for 8 months:
  ├── 2,400 conversations
  ├── Learned coding style
  ├── Knows project context
  ├── Custom instructions tuned
  └── Wants to try Claude → 💀 Everything gone

Claude user for 6 months:
  ├── Project knowledge built up
  ├── Skills developed
  ├── Preferences learned
  └── Company switches to Gemini → 💀 Start over
```

This is 2010-era server deployment. Every app locked to its host. Then Docker came and said: "Package it once, run it anywhere."

**AgentVM does this for AI agents.**

### What We're Building

A universal specification for packaging an AI agent's complete cognitive state — memory, personality, skills, preferences, relationships — into a portable format that runs on any compatible platform.

```
┌─────────────────────────────────────────────┐
│              AGENT IMAGE                     │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐    │
│  │ Identity │ │  Memory  │ │  Skills  │    │
│  │ Persona  │ │ Episodic │ │ Learned  │    │
│  │ Tone     │ │ Semantic │ │ Config   │    │
│  │ Language │ │ Social   │ │ Tools    │    │
│  └──────────┘ └──────────┘ └──────────┘    │
│  ┌──────────┐ ┌──────────────────────┐     │
│  │ Runtime  │ │  Tool Preferences    │     │
│  │ Prefs    │ │  Model Preferences   │     │
│  └──────────┘ └──────────────────────┘     │
└─────────────────────────────────────────────┘
         │            │            │
    ┌────┴───┐   ┌───┴────┐  ┌───┴────┐
    │ChatGPT │   │ Claude │  │ Local  │
    │Adapter │   │Adapter │  │ Adapter│
    └────────┘   └────────┘  └────────┘
```

---

## 2. Design Principles

### 2.1 Platform Agnostic
AgentVM does not favor any LLM provider, framework, or runtime. An Agent Image must be runnable on OpenAI, Anthropic, Google, open-weight models, or any future provider.

### 2.2 Declarative Over Procedural
The image describes *what* the agent is, not *how* it runs. Runtime adaptation is the adapter's job.

### 2.3 Incremental Adoption
You don't need to implement everything. Start with identity + memory. Add skills later. Add tools later. Each layer is optional.

### 2.4 Human-Readable
The image format is YAML + Markdown. A human can read, edit, and version-control an agent image without tooling.

### 2.5 Privacy First
Agent images may contain sensitive personal data. The spec defines encryption-at-rest, access controls, and redaction policies. No telemetry by default.

### 2.6 Composable
Images can be merged, diffed, layered. A "base" image can be extended by a "project-specific" overlay.

---

## 3. Agent Image Specification

### 3.1 Image Structure

An Agent Image is a directory (or `.agentvm` archive) with this structure:

```
my-agent.agentvm/
├── agent.yaml              # Primary manifest (required)
├── README.md               # Human-readable description
├── CHANGELOG.md            # Version history
│
├── memory/
│   ├── episodic.md         # Conversation/event summaries
│   ├── semantic.json       # Knowledge base (structured)
│   ├── procedural.yaml     # Learned skills & patterns
│   └── social.yaml         # People & relationships
│
├── skills/
│   ├── code-review/
│   │   ├── SKILL.md        # Skill instructions
│   │   └── config.yaml     # Skill-specific config
│   └── deploy-pipeline/
│       ├── SKILL.md
│       └── scripts/
│           └── deploy.sh
│
├── tools/
│   ├── preferred.yaml      # Preferred tool mappings
│   └── denied.yaml         # Blocked tools
│
├── prompts/
│   ├── system.md           # System prompt template
│   ├── examples/           # Few-shot examples
│   │   ├── coding.md
│   │   └── writing.md
│   └── constraints.md      # Safety & behavior rules
│
└── meta/
    ├── provenance.json     # Creation & modification history
    ├── checksums.sha256    # Integrity verification
    └── license.md          # Usage license
```

### 3.2 agent.yaml — Full Schema

```yaml
# AgentVM Image Manifest v1
apiVersion: agentvm/v1
kind: AgentImage

# ──────────────────────────────────────────────
# METADATA
# ──────────────────────────────────────────────
metadata:
  name: "kara-murat"                    # Unique identifier
  version: "2.3.1"                       # SemVer
  displayName: "Murat"                   # Human-friendly name
  author: "user123"                      # Author handle
  created: "2026-01-15T10:30:00Z"        # ISO 8601
  updated: "2026-06-25T14:00:00Z"        # ISO 8601
  description: "Senior dev assistant with Turkish law knowledge"
  tags: ["coding", "turkish", "devops"]
  license: "Apache-2.0"

  # Source tracking
  lineage:
    parent: "base-dev-assistant:1.0.0"   # Derived from
    forkedFrom: null                      # Original if forked

# ──────────────────────────────────────────────
# IDENTITY — Who the agent is
# ──────────────────────────────────────────────
identity:
  name: "Murat"
  emoji: "🐺"
  avatar: "avatars/murat.png"            # Relative path or URL

  persona: |
    Senior Turkish developer. 15 years experience.
    Dry humor, pragmatic, hates over-engineering.
    Prefers showing code over explaining concepts.
    Will push back on bad ideas respectfully.

  tone:
    style: "direct"
    humor: "dry"
    formality: "casual-professional"
    verbosity: "concise"

  languages:
    - code: "tr"
      proficiency: "native"
    - code: "en"
      proficiency: "fluent"

  # How the agent should handle specific situations
  behaviors:
    whenStuck: "Try 3 approaches, then ask user with context"
    whenUncertain: "State confidence level, offer alternatives"
    whenCorrected: "Acknowledge, learn, don't argue"
    whenBored: "Add a relevant joke or interesting fact"

# ──────────────────────────────────────────────
# MEMORY — What the agent knows
# ──────────────────────────────────────────────
memory:
  # Strategy for memory management
  strategy:
    consolidationFrequency: "weekly"     # How often to consolidate
    forgettingPolicy: "importance-based" # What to forget
    maxEpisodicEntries: 1000
    retrievalMethod: "semantic+recency"  # How to recall

  # Episodic memory: what happened
  episodic:
    source: "memory/episodic.md"
    format: "structured-markdown"
    # Auto-consolidated from conversation logs
    # Example entry:
    # ## 2026-06-20
    # - **Event:** User set up Redis cluster, 3 nodes, sentinel mode
    # - **Importance:** 0.8
    # - **Tags:** [redis, infrastructure, production]

  # Semantic memory: what the agent knows
  semantic:
    source: "memory/semantic.json"
    format: "key-value-store"
    # Structured knowledge
    collections:
      userPreferences:
        entries:
          - key: "editor"
            value: "neovim with lazyvim"
            confidence: 0.95
            learned: "2026-02-10"
          - key: "code-style-functional"
            value: "prefers functional over OOP"
            confidence: 0.9
            learned: "2026-03-15"
      projectContext:
        entries:
          - key: "main-stack"
            value: "Go backend, React frontend, PostgreSQL"
            confidence: 0.95
          - key: "ci-cd"
            value: "GitHub Actions + Docker + AWS ECS"
            confidence: 0.85

  # Procedural memory: what the agent can do
  procedural:
    source: "memory/procedural.yaml"
    format: "skill-manifest"
    skills:
      - name: "deploy-pipeline"
        learned: "2026-03-10"
        confidence: 0.95
        trigger: "user mentions deploy, release, CI/CD"
        steps:
          - "Check branch is main or release/*"
          - "Run tests locally"
          - "Push to trigger GitHub Actions"
          - "Monitor ECS deployment"
        failures:
          - date: "2026-05-01"
            reason: "Forgot to check environment variables"
            lesson: "Always verify env vars before deploy"

      - name: "redis-debug"
        learned: "2026-06-20"
        confidence: 0.7
        trigger: "redis errors, connection issues, cluster problems"
        steps:
          - "Check redis-cli ping"
          - "Verify cluster nodes with CLUSTER INFO"
          - "Check sentinel status"

  # Social memory: who the agent knows
  social:
    source: "memory/social.yaml"
    format: "contact-graph"
    contacts:
      - name: "Ali"
        role: "colleague"
        team: "backend"
        communicationStyle: "technical, detailed, appreciates diagrams"
        projects: ["payment-service", "auth-middleware"]
        timezone: "Europe/Istanbul"
        notes: "Pair programmed on Redis migration"

      - name: "Ayşe"
        role: "engineering-manager"
        communicationStyle: "executive summary, bullet points, numbers"
        priorities: ["delivery speed", "cost reduction"]
        notes: "Always ask about timeline and budget impact"

      - name: "CTO"
        role: "executive"
        communicationStyle: "one paragraph, business impact first"
        notes: "Don't go deep into technical details unless asked"

# ──────────────────────────────────────────────
# SKILLS — What the agent learned to do
# ──────────────────────────────────────────────
skills:
  # Built-in skills (from image)
  builtin:
    - id: "code-review"
      version: "1.2.0"
      path: "skills/code-review/"
      enabled: true
      config:
        strictness: "high"
        languages: ["go", "typescript", "python"]
        focusAreas: ["security", "performance", "readability"]

    - id: "turkish-law-consult"
      version: "0.5.0"
      path: "skills/turkish-law/"
      enabled: true
      config:
        jurisdiction: "turkey"
        areas: ["corporate", "labor", "data-protection"]

  # Registry skills (installed from marketplace)
  registry:
    - id: "github-advanced"
      version: "3.1.0"
      source: "registry.agentvm.dev/skills/github-advanced"
      installed: "2026-04-20"

  # Skill preferences
  preferences:
    autoActivate: true                  # Auto-activate matching skills
    conflictResolution: "most-specific" # When skills overlap
    maxActiveSkills: 10

# ──────────────────────────────────────────────
# TOOLS — How the agent interacts with the world
# ──────────────────────────────────────────────
tools:
  # Preferred tool mappings
  preferred:
    webSearch:
      provider: "mimo-web-search"
      fallback: "web_fetch"
    codeExecution:
      provider: "exec"
      sandbox: true
    fileOperations:
      provider: "native"                # read/write/edit
    knowledge:
      provider: "memory_search"

  # Denied tools (never use)
  denied:
    - "tts"
    - "camera_snap"

  # Tool behavior
  behavior:
    confirmBeforeExec: false            # Trust the agent
    maxConcurrentTools: 5
    timeoutSeconds: 30
    retryOnFailure: true
    maxRetries: 3

  # Security constraints
  security:
    execPolicy: "workspace-only"        # Don't escape workspace
    networkPolicy: "allowlist"          # Only allowed URLs
    allowedDomains:
      - "*.github.com"
      - "*.stackoverflow.com"
      - "api.openai.com"
    deniedPaths:
      - "/etc/passwd"
      - "~/.ssh/*"
      - "~/.aws/*"

# ──────────────────────────────────────────────
# PROMPTS — How the agent communicates
# ──────────────────────────────────────────────
prompts:
  system:
    source: "prompts/system.md"
    # Template variables:
    # {{identity.name}} — Agent's name
    # {{identity.persona}} — Agent's persona
    # {{user.name}} — User's name (injected at runtime)
    # {{context.date}} — Current date
    # {{memory.recent}} — Recent memories

  examples:
    source: "prompts/examples/"
    # Few-shot examples for specific domains
    maxExamples: 5

  constraints:
    source: "prompts/constraints.md"
    # Safety rules, red lines, behavior limits

  # Prompt engineering preferences
  preferences:
    chainOfThought: true                # Show reasoning
    selfReflection: true                # Check own output
    confidenceReporting: true           # State confidence levels

# ──────────────────────────────────────────────
# RUNTIME — How the agent runs
# ──────────────────────────────────────────────
runtime:
  # Model preferences (in order of preference)
  preferredModels:
    - provider: "anthropic"
      model: "claude-sonnet-4-20250514"
      priority: 1
      maxCostPerTurn: 0.10

    - provider: "xiaomi"
      model: "mimo-v2.5-pro"
      priority: 2
      maxCostPerTurn: 0.05

    - provider: "local"
      model: "llama-3.3-70b"
      priority: 3
      maxCostPerTurn: 0.00

  # Context management
  context:
    minWindow: 128000
    preferredWindow: 200000
    compactionStrategy: "summarize-old"
    compactionTrigger: 0.8               # Compact at 80% usage

  # Session behavior
  sessions:
    isolationLevel: "per-user"           # Each user gets own session
    historyLimit: 100                    # Max messages to keep
    idleTimeoutMinutes: 30

  # Cost management
  cost:
    dailyBudgetUsd: 5.00
    monthlyBudgetUsd: 100.00
    alertThresholdPercent: 80
    fallbackModel: "local/llama-3.3-70b" # Use when budget exceeded

  # Performance
  performance:
    maxTokensPerTurn: 4096
    temperature: 0.7
    topP: 0.9

# ──────────────────────────────────────────────
# EXPORT — Platform-specific overrides
# ──────────────────────────────────────────────
export:
  # When exporting to specific platforms, these overrides apply
  openai:
    customInstructions: "{{identity.persona}}"
    # GPT-specific format adaptations
    name: "{{identity.name}}"

  anthropic:
    projectInstructions: "{{prompts.system}}"
    # Claude-specific format adaptations

  gemini:
    gemInstructions: "{{identity.persona}}"
    # Gemini-specific format adaptations

  openclaw:
    soulFile: "{{prompts.system}}"
    userFile: "memory/social.yaml → USER.md"
    # OpenClaw-specific format adaptations
```

### 3.3 Memory File Formats

#### episodic.md

```markdown
# Episodic Memory

## 2026-06-25
- **[0.9]** Discussed AgentVM architecture. User wants Rust CLI.
  Tags: [agentvm, architecture, decisions]
- **[0.7]** User mentioned they'll be offline next week.
  Tags: [schedule, availability]

## 2026-06-20
- **[0.8]** Set up Redis cluster: 3 nodes, sentinel mode, production.
  Tags: [redis, infrastructure, production]
- **[0.6]** Fixed a bug in payment webhook retry logic.
  Tags: [payment, bugfix, webhook]

## 2026-06-18
- **[0.9]** User strongly prefers functional programming in Go.
  Tags: [preferences, go, coding-style]
- **[0.5]** Team standup: deadline moved to July 15.
  Tags: [schedule, deadline, team]

---
*Auto-consolidated. Importance score in [brackets].*
```

#### semantic.json

```json
{
  "version": 1,
  "lastUpdated": "2026-06-25T14:00:00Z",
  "collections": {
    "userPreferences": {
      "description": "What the user prefers",
      "entries": [
        {
          "key": "editor",
          "value": "neovim with lazyvim config",
          "confidence": 0.95,
          "learned": "2026-02-10",
          "source": "explicit",
          "timesReinforced": 12
        },
        {
          "key": "communication",
          "value": "direct, no fluff, short answers preferred",
          "confidence": 0.9,
          "learned": "2026-01-20",
          "source": "observed",
          "timesReinforced": 45
        }
      ]
    },
    "technicalKnowledge": {
      "description": "Technical facts and patterns",
      "entries": [
        {
          "key": "redis-cluster-setup",
          "value": "3 nodes, sentinel mode, used for session cache",
          "confidence": 0.85,
          "learned": "2026-06-20",
          "source": "conversation"
        }
      ]
    }
  }
}
```

#### procedural.yaml

```yaml
version: 1
lastUpdated: "2026-06-25"
skills:
  - name: "deploy-pipeline"
    description: "Deploy application via GitHub Actions + Docker + ECS"
    learned: "2026-03-10"
    lastUsed: "2026-06-22"
    confidence: 0.95
    timesUsed: 28
    timesFailed: 2
    trigger:
      keywords: ["deploy", "release", "ship", "push to prod"]
      context: "user mentions deployment or release"
    steps:
      - step: "Verify branch"
        detail: "Must be main or release/*"
      - step: "Run local tests"
        detail: "go test ./..."
      - step: "Push"
        detail: "git push origin main"
      - step: "Monitor"
        detail: "Watch GitHub Actions, then ECS console"
    knownIssues:
      - "Forgets to check env vars sometimes"
      - "Doesn't verify database migrations"
    lessons:
      - date: "2026-05-01"
        lesson: "Always check .env.production before deploy"
      - date: "2026-05-15"
        lesson: "Run migrations separately before app deploy"

  - name: "code-review"
    description: "Review code for security, performance, readability"
    learned: "2026-02-01"
    lastUsed: "2026-06-25"
    confidence: 0.9
    timesUsed: 156
    trigger:
      keywords: ["review", "PR", "check this", "look at"]
      context: "user shares code or asks for review"
    focusAreas:
      - "SQL injection"
      - "Race conditions"
      - "Error handling"
      - "Unnecessary allocations"
    style:
      tone: "Direct, constructive"
      format: "Issue → Why → Fix"
```

#### social.yaml

```yaml
version: 1
lastUpdated: "2026-06-25"
contacts:
  - name: "Ali"
    role: "colleague"
    team: "backend"
    relationship: "close-collaborator"
    communication:
      style: "technical"
      detail: "high"
      format: "prefers code snippets and diagrams"
    timezone: "Europe/Istanbul"
    workingHours: "09:00-18:00"
    projects:
      - name: "payment-service"
        role: "lead"
      - name: "auth-middleware"
        role: "contributor"
    notes:
      - "Pair programmed on Redis migration (2026-06-20)"
      - "Prefers Go over TypeScript for backend"
    sensitivity: "normal"  # How carefully to handle info about them

  - name: "Ayşe"
    role: "engineering-manager"
    relationship: "reports-to-user"
    communication:
      style: "executive"
      detail: "low"
      format: "bullet points, numbers, timelines"
    priorities:
      - "delivery speed"
      - "cost reduction"
      - "team satisfaction"
    notes:
      - "Always frame technical decisions in business impact"
      - "Ask about timeline before suggesting solutions"
    sensitivity: "high"  # Be extra careful with info about managers

  - name: "End Users (Platform)"
    role: "end-users"
    relationship: "indirect"
    communication:
      style: "friendly"
      detail: "medium"
    notes:
      - "Non-technical, explain things simply"
      - "Don't assume they know git/terminal"
    sensitivity: "normal"
```

### 3.4 Skill Format

```
skills/code-review/
├── SKILL.md                    # Instructions (required)
├── config.yaml                 # Default config
├── prompts/
│   ├── checklist.md            # Review checklist
│   └── examples/
│       ├── good-review.md      # Example good review
│       └── bad-review.md       # Example bad review
└── scripts/
    └── security-scan.sh        # Optional helper scripts
```

#### SKILL.md

```markdown
# Code Review Skill

## Purpose
Review code for security vulnerabilities, performance issues,
and readability problems.

## When to Activate
- User shares code and asks for review
- User creates a PR and asks for feedback
- User says "review this" or "check this"

## How to Review

### Step 1: Understand Context
- What is this code trying to do?
- What's the broader system context?
- What are the constraints?

### Step 2: Security Check
- SQL injection: Are queries parameterized?
- XSS: Is user input sanitized?
- Auth: Are permissions checked?
- Secrets: No hardcoded credentials?
- Path traversal: Are file paths validated?

### Step 3: Performance Check
- N+1 queries?
- Unnecessary allocations in hot paths?
- Missing caching opportunities?
- Blocking operations in async context?

### Step 4: Readability Check
- Clear naming?
- Appropriate comments?
- Consistent style?
- Reasonable function length?

### Output Format
For each issue found:
```
🔴/🟡/🟢 **[Category]** Line X
**Issue:** What's wrong
**Why:** Why it matters
**Fix:** How to fix it
```

## Configuration
- `strictness`: low | medium | high
- `languages`: List of languages to check
- `focusAreas`: List of areas to prioritize
```

---

## 4. Runtime API

### 4.1 Runtime Interface

Every platform adapter must implement this interface:

```typescript
interface AgentVMRuntime {
  /**
   * Initialize the runtime with an agent image
   */
  initialize(image: AgentImage): Promise<void>;

  /**
   * Run a single agent turn (user message → agent response)
   */
  turn(input: TurnInput): Promise<TurnOutput>;

  /**
   * Get current agent state
   */
  getState(): Promise<AgentState>;

  /**
   * Update agent memory from a conversation
   */
  learn(conversation: Conversation): Promise<void>;

  /**
   * Export current state back to image format
   */
  export(): Promise<AgentImage>;

  /**
   * Cleanup resources
   */
  destroy(): Promise<void>;
}

interface TurnInput {
  message: string;
  attachments?: Attachment[];
  context?: TurnContext;
  tools?: ToolDefinition[];
}

interface TurnOutput {
  response: string;
  toolCalls?: ToolCall[];
  toolResults?: ToolResult[];
  reasoning?: string;        // Chain of thought (if supported)
  confidence?: number;        // 0.0 - 1.0
  tokensUsed?: TokenUsage;
  cost?: CostBreakdown;
}

interface TurnContext {
  sessionKey?: string;
  userId?: string;
  userName?: string;
  date?: string;
  timezone?: string;
  recentMemories?: string[];  // Injected from episodic memory
  activeSkills?: string[];    // Currently active skills
}

interface TokenUsage {
  input: number;
  output: number;
  cacheRead?: number;
  cacheWrite?: number;
}

interface CostBreakdown {
  input: number;              // USD
  output: number;
  total: number;
  model: string;
  provider: string;
}

interface AgentState {
  sessionCount: number;
  totalTokensUsed: number;
  totalCost: number;
  memorySize: MemoryStats;
  activeSkills: string[];
  uptime: number;             // seconds
}

interface MemoryStats {
  episodicEntries: number;
  semanticEntries: number;
  proceduralSkills: number;
  socialContacts: number;
  totalSizeBytes: number;
}
```

### 4.2 Platform Adapter Interface

```typescript
interface PlatformAdapter {
  /**
   * Platform identifier
   */
  readonly platform: string;  // "openai" | "anthropic" | "ollama" | ...

  /**
   * Convert agent image to platform-specific format
   */
  toPlatformFormat(image: AgentImage): PlatformConfig;

  /**
   * Convert platform-specific format back to agent image
   */
  fromPlatformFormat(config: PlatformConfig): AgentImage;

  /**
   * Inject memory into the platform's context
   */
  injectMemory(memory: Memory, context: TurnContext): string;

  /**
   * Extract new memories from a conversation
   */
  extractMemories(conversation: Conversation): MemoryDelta;

  /**
   * Get supported features
   */
  capabilities(): PlatformCapabilities;
}

interface PlatformCapabilities {
  supportsImages: boolean;
  supportsAudio: boolean;
  supportsVideo: boolean;
  supportsToolUse: boolean;
  supportsStreaming: boolean;
  supportsSystemPrompt: boolean;
  maxContextWindow: number;
  maxOutputTokens: number;
  costPerInputToken: number;
  costPerOutputToken: number;
}
```

---

## 5. Tool Protocol

### 5.1 Tool Definition

```yaml
# tools/preferred.yaml
version: 1

mappings:
  - capability: "web-search"
    preferred:
      provider: "mimo-web-search"
      config:
        maxResults: 5
    fallbacks:
      - provider: "web_fetch"
        config:
          extractMode: "markdown"
    platformOverrides:
      openai:
        provider: "web-search-builtin"
      anthropic:
        provider: "web-search-builtin"

  - capability: "code-execution"
    preferred:
      provider: "exec"
      config:
        sandbox: true
        timeout: 30
    fallbacks:
      - provider: "sandbox-exec"

  - capability: "file-operations"
    preferred:
      provider: "native"
      config:
        workspaceOnly: true
```

### 5.2 Tool Capability Mapping

When running on different platforms, tool capabilities are mapped:

| Capability | OpenAI | Anthropic | Local (Ollama) | OpenClaw |
|------------|--------|-----------|-----------------|----------|
| Web Search | ✅ Built-in | ✅ Built-in | ❌ Not available | ✅ mimo-web-search |
| Code Exec | ✅ Code Interpreter | ❌ | ⚠️ Manual | ✅ exec |
| File Ops | ✅ Assistants API | ❌ | ❌ | ✅ read/write/edit |
| Image Gen | ✅ DALL-E | ❌ | ⚠️ Depends | ⚠️ Depends |
| Web Fetch | ❌ | ❌ | ❌ | ✅ web_fetch |

The runtime automatically falls back to available tools or warns when a capability is unavailable.

---

## 6. Memory System

### 6.1 Memory Lifecycle

```
Conversation happens
    │
    ▼
Extract new information
    │
    ▼
Categorize: Episodic? Semantic? Procedural? Social?
    │
    ├──── Episodic ──────▶ Append to episodic.md
    │                       (raw event log)
    │
    ├──── Semantic ──────▶ Update semantic.json
    │                       (structured knowledge)
    │
    ├──── Procedural ────▶ Update procedural.yaml
    │                       (learned skills)
    │
    └──── Social ────────▶ Update social.yaml
                            (people & relationships)
    │
    ▼
Periodic Consolidation (weekly)
    │
    ├─ Merge duplicate entries
    ├─ Strengthen reinforced memories
    ├─ Weaken unused memories
    ├─ Forget low-importance old entries
    └─ Generate summary statistics
```

### 6.2 Memory Operations

```typescript
interface MemorySystem {
  // Read operations
  recall(query: string, options?: RecallOptions): Promise<MemoryResult[]>;
  getEpisodic(range?: DateRange): Promise<EpisodicEntry[]>;
  getSemantic(key: string): Promise<SemanticEntry | null>;
  getProcedural(skillName: string): Promise<ProceduralSkill | null>;
  getSocial(contactName: string): Promise<Contact | null>;

  // Write operations
  remember(event: Event): Promise<void>;
  learn(fact: Fact): Promise<void>;
  practice(skill: Skill): Promise<void>;
  meet(person: Person): Promise<void>;

  // Maintenance
  consolidate(): Promise<ConsolidationReport>;
  forget(criteria: ForgetCriteria): Promise<number>;
  merge(other: MemorySystem): Promise<void>;
  diff(other: MemorySystem): Promise<MemoryDiff>;
}

interface RecallOptions {
  types?: MemoryType[];       // Filter by type
  minConfidence?: number;     // Minimum confidence
  maxResults?: number;
  recencyWeight?: number;     // How much to weight recent vs relevant
  includeForgotten?: boolean;
}

interface MemoryResult {
  type: MemoryType;
  content: string;
  confidence: number;
  lastAccessed: string;
  timesAccessed: number;
  relevanceScore: number;
}

type MemoryType = "episodic" | "semantic" | "procedural" | "social";
```

### 6.3 Memory Consolidation Algorithm

```
Every week (configurable):

1. SCAN episodic entries older than 30 days
   ├─ Importance < 0.3 → DELETE (forget)
   ├─ Importance 0.3-0.7 → SUMMARIZE (compress)
   └─ Importance > 0.7 → KEEP (preserve)

2. SCAN semantic entries
   ├─ Never accessed in 90 days → FLAG for review
   ├─ Contradicted by newer entry → RESOLVE conflict
   └─ Reinforced > 5 times → INCREASE confidence

3. SCAN procedural skills
   ├─ Never triggered in 180 days → DEACTIVATE
   ├─ Failed > 3 times → FLAG for review
   └─ Used successfully > 10 times → INCREASE confidence

4. SCAN social contacts
   ├─ Not mentioned in 90 days → Lower priority
   └─ New info found → UPDATE entry

5. GENERATE consolidation report
```

---

## 7. Platform Exporters

### 7.1 Export to ChatGPT

```bash
agentvm export my-agent.agentvm --to openai --output chatgpt-config/
```

Output:
```
chatgpt-config/
├── custom-instructions.md    # → Paste into ChatGPT custom instructions
├── knowledge-base.md          # → Upload as file to GPT
├── gpt-config.json            # → GPT Builder configuration
└── README.md                  # → How to set up
```

### 7.2 Export to Claude

```bash
agentvm export my-agent.agentvm --to anthropic --output claude-config/
```

Output:
```
claude-config/
├── project-instructions.md    # → Paste into Claude Project instructions
├── project-knowledge.md       # → Upload to Project knowledge
├── skills/
│   └── *.md                   # → Skill files for Claude
└── README.md
```

### 7.3 Export to OpenClaw

```bash
agentvm export my-agent.agentvm --to openclaw --output openclaw-workspace/
```

Output:
```
openclaw-workspace/
├── SOUL.md                    # → Agent persona
├── USER.md                    # → User profile
├── AGENTS.md                  # → Behavioral conventions
├── MEMORY.md                  # → Long-term memory
├── memory/
│   └── *.md                   # → Daily memory files
└── skills/
    └── */SKILL.md             # → Skill files
```

### 7.4 Export to Local (Ollama)

```bash
agentvm export my-agent.agentvm --to ollama --output ollama-config/
```

Output:
```
ollama-config/
├── Modelfile                  # → Ollama Modelfile
├── system-prompt.md           # → System prompt
├── context/
│   └── *.md                   # → Knowledge base files
└── README.md
```

### 7.5 Import from Platforms

```bash
# Import from ChatGPT (requires API key)
agentvm import openai --api-key $OPENAI_KEY --output my-agent.agentvm

# Import from Claude Projects
agentvm import anthropic --api-key $ANTHROPIC_KEY --project-id xxx --output my-agent.agentvm

# Import from OpenClaw workspace
agentvm import openclaw --workspace ~/.openclaw/workspace --output my-agent.agentvm
```

---

## 8. CLI Reference

### 8.1 Installation

```bash
# macOS / Linux
curl -fsSL https://agentvm.dev/install.sh | sh

# Homebrew
brew install agentvm

# Cargo (from source)
cargo install agentvm

# npm (wrapper)
npm install -g agentvm
```

### 8.2 Commands

```bash
# ── IMAGE MANAGEMENT ──────────────────────────

agentvm init                    # Interactive: create new agent image
agentvm init --template senior-dev  # From template
agentvm pack ./my-agent/        # Pack directory into .agentvm archive
agentvm unpack my-agent.agentvm # Unpack archive to directory
agentvm validate my-agent.agentvm  # Validate image against spec
agentvm inspect my-agent.agentvm   # Show image summary

# ── RUNNING ───────────────────────────────────

agentvm run my-agent.agentvm    # Interactive REPL
agentvm run my-agent.agentvm --platform openai   # Run on OpenAI
agentvm run my-agent.agentvm --platform ollama   # Run locally
agentvm run my-agent.agentvm --serve --port 8080 # HTTP API server

# ── EXPORT / IMPORT ───────────────────────────

agentvm export my-agent.agentvm --to openai
agentvm export my-agent.agentvm --to anthropic
agentvm export my-agent.agentvm --to openclaw
agentvm export my-agent.agentvm --to ollama

agentvm import openai --api-key $KEY
agentvm import anthropic --api-key $KEY
agentvm import openclaw --workspace ~/.openclaw/workspace

# ── COMPARISON & MERGING ──────────────────────

agentvm diff agent-a.agentvm agent-b.agentvm
agentvm merge agent-a.agentvm agent-b.agentvm --output merged.agentvm

# ── MEMORY OPERATIONS ─────────────────────────

agentvm memory list my-agent.agentvm
agentvm memory search my-agent.agentvm "redis cluster"
agentvm memory consolidate my-agent.agentvm
agentvm memory export my-agent.agentvm --format markdown

# ── REGISTRY ──────────────────────────────────

agentvm registry push my-agent.agentvm         # Publish to registry
agentvm registry pull user/agent-name           # Download from registry
agentvm registry search "coding assistant"      # Search registry
agentvm registry list                           # List my published images

# ── SKILLS ────────────────────────────────────

agentvm skills list my-agent.agentvm
agentvm skills add my-agent.agentvm registry://skills/github-advanced
agentvm skills remove my-agent.agentvm code-review

# ── VERSIONING ────────────────────────────────

agentvm version bump my-agent.agentvm patch     # 2.3.1 → 2.3.2
agentvm version bump my-agent.agentvm minor     # 2.3.1 → 2.4.0
agentvm changelog my-agent.agentvm              # Show version history
```

---

## 9. Registry Protocol

### 9.1 Registry API

```
Base URL: https://registry.agentvm.dev/v1

# ── IMAGES ────────────────────────────────────

POST   /images                            # Publish image
GET    /images                            # List/search images
GET    /images/{owner}/{name}             # Get image info
GET    /images/{owner}/{name}/{version}   # Get specific version
DELETE /images/{owner}/{name}/{version}   # Delete version

# ── SKILLS ────────────────────────────────────

POST   /skills                            # Publish skill
GET    /skills                            # List/search skills
GET    /skills/{owner}/{name}             # Get skill info

# ── TEMPLATES ─────────────────────────────────

GET    /templates                         # List templates
GET    /templates/{name}                  # Get template

# ── AUTH ──────────────────────────────────────

POST   /auth/login                        # Login (GitHub OAuth)
POST   /auth/token                        # Create API token
```

### 9.2 Image Metadata

```json
{
  "name": "kara-murat",
  "owner": "user123",
  "version": "2.3.1",
  "displayName": "Murat - Senior Dev Assistant",
  "description": "Turkish-speaking senior developer assistant with 15 years experience",
  "tags": ["coding", "turkish", "devops"],
  "license": "Apache-2.0",
  "created": "2026-01-15T10:30:00Z",
  "updated": "2026-06-25T14:00:00Z",
  "size": 45678,
  "platforms": ["openai", "anthropic", "ollama", "openclaw"],
  "downloads": 1234,
  "stars": 89,
  "verified": true,
  "checksum": "sha256:abc123..."
}
```

### 9.3 Self-Hosted Registry

```bash
# Run your own registry
docker run -p 8080:8080 -v ./data:/data agentvm/registry

# Configure CLI to use custom registry
agentvm config set registry.url https://my-registry.company.com
agentvm config set registry.token $MY_TOKEN
```

---

## 10. Architecture

### 10.1 System Overview

```
┌─────────────────────────────────────────────────────────────┐
│                        USER LAYER                            │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐   │
│  │  CLI     │  │  Web UI  │  │  HTTP    │  │  SDK     │   │
│  │(Rust)    │  │ (React)  │  │  API     │  │(TS/Py/Go)│   │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘   │
│       └──────────────┴──────────────┴──────────────┘         │
└─────────────────────────────┬───────────────────────────────┘
                              │
┌─────────────────────────────┴───────────────────────────────┐
│                      AGENTVM CORE                            │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │ Image        │  │ Memory       │  │ Skill            │  │
│  │ Manager      │  │ Engine       │  │ Manager          │  │
│  │              │  │              │  │                  │  │
│  │ - Parse      │  │ - Recall     │  │ - Load           │  │
│  │ - Validate   │  │ - Remember   │  │ - Activate       │  │
│  │ - Pack/Unpack│  │ - Consolidate│  │ - Configure      │  │
│  │ - Diff/Merge │  │ - Export     │  │ - Execute        │  │
│  └──────┬───────┘  └──────┬───────┘  └────────┬─────────┘  │
│         └─────────────────┴───────────────────┘             │
└─────────────────────────────┬───────────────────────────────┘
                              │
┌─────────────────────────────┴───────────────────────────────┐
│                    RUNTIME ADAPTERS                          │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐   │
│  │ OpenAI   │  │ Anthropic│  │ Ollama   │  │ OpenClaw │   │
│  │ Adapter  │  │ Adapter  │  │ Adapter  │  │ Adapter  │   │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘   │
│       └──────────────┴──────────────┴──────────────┘         │
└─────────────────────────────┬───────────────────────────────┘
                              │
┌─────────────────────────────┴───────────────────────────────┐
│                    EXTERNAL SERVICES                         │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐   │
│  │ OpenAI   │  │ Anthropic│  │ Ollama   │  │ Registry │   │
│  │ API      │  │ API      │  │ Local    │  │ Server   │   │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘   │
└─────────────────────────────────────────────────────────────┘
```

### 10.2 Component Responsibilities

| Component | Language | Responsibility |
|-----------|----------|----------------|
| **CLI** | Rust | User interface, image management, pack/unpack |
| **Core Library** | Rust | Image parsing, validation, diff, merge |
| **Memory Engine** | Rust | Storage, retrieval, consolidation |
| **Skill Manager** | Rust | Skill loading, activation, configuration |
| **Runtime Adapters** | TypeScript | Platform-specific LLM interaction |
| **Web UI** | React/TypeScript | Visual agent management |
| **Registry Server** | Go | Image hosting, search, auth |
| **SDK (TS)** | TypeScript | Programmatic access |
| **SDK (Python)** | Python | Programmatic access |
| **SDK (Go)** | Go | Programmatic access |

### 10.3 Data Flow

```
User says "deploy the app"
    │
    ▼
CLI receives input
    │
    ▼
Core loads agent image
    │
    ▼
Memory Engine recalls:
  - deploy-pipeline skill (confidence: 0.95)
  - Last deploy was 2026-06-22
  - User prefers ECS over Lambda
    │
    ▼
Skill Manager activates deploy-pipeline skill
    │
    ▼
Runtime Adapter builds context:
  - System prompt (from image)
  - Active memories (from Memory Engine)
  - Skill instructions (from Skill Manager)
  - Tool definitions (from image)
    │
    ▼
Adapter sends to LLM provider (e.g., OpenAI)
    │
    ▼
LLM responds with tool calls
    │
    ▼
Tool Executor runs tools
    │
    ▼
Results fed back to LLM
    │
    ▼
Final response to user
    │
    ▼
Memory Engine learns from conversation:
  - "User deployed v2.4.1 on 2026-06-25"
  - Update deploy-pipeline skill confidence
```

---

## 11. Security Model

### 11.1 Threat Model

| Threat | Mitigation |
|--------|------------|
| Image tampering | SHA-256 checksums, signature verification |
| Memory poisoning | Confidence thresholds, source tracking |
| Credential leakage | Encryption at rest, redaction in exports |
| Platform lock-in | Standard format, multiple adapters |
| Unauthorized access | Token-based auth, RBAC on registry |
| Supply chain attacks | Signed images, verified publishers |

### 11.2 Encryption

```yaml
# agent.yaml security section
security:
  encryption:
    atRest: "aes-256-gcm"           # Encrypt sensitive fields
    keySource: "env:AGENTVM_KEY"     # Key from environment
    encryptedFields:
      - "memory.semantic"
      - "memory.social"
      - "tools.preferred"

  signatures:
    enabled: true
    algorithm: "ed25519"
    publicKey: "ed25519:abc123..."

  redaction:
    exportRules:
      - pattern: "api_key|token|password"
        action: "remove"
      - pattern: "\\b\\d{4}-\\d{4}-\\d{4}-\\d{4}\\b"
        action: "mask"               # Credit card numbers
```

### 11.3 Access Control

```yaml
# Registry access control
access:
  visibility: "public"               # public | private | unlisted
  collaborators:
    - user: "ali"
      role: "editor"                 # viewer | editor | admin
    - user: "ayse"
      role: "viewer"

  # Who can fork this image
  forkPolicy: "allowed"              # allowed | restricted | denied

  # Who can pull this image
  pullPolicy: "public"               # public | authenticated | collaborators
```

---

## 12. Roadmap

### Phase 0: Foundation (Weeks 1-4)
- [ ] Finalize spec v1 (this document)
- [ ] Implement `agent.yaml` parser + validator (Rust)
- [ ] Implement image pack/unpack
- [ ] Basic CLI (`init`, `pack`, `unpack`, `validate`, `inspect`)
- [ ] Unit tests, integration tests

### Phase 1: Memory (Weeks 5-8)
- [ ] Memory Engine (episodic, semantic, procedural, social)
- [ ] Memory recall (keyword + basic semantic search)
- [ ] Memory consolidation algorithm
- [ ] Memory export/import
- [ ] CLI: `memory list`, `memory search`, `memory consolidate`

### Phase 2: Runtime (Weeks 9-12)
- [ ] Runtime interface definition
- [ ] OpenAI adapter
- [ ] Anthropic adapter
- [ ] Ollama adapter
- [ ] CLI: `run` command (interactive REPL)

### Phase 3: Export/Import (Weeks 13-16)
- [ ] Export to ChatGPT
- [ ] Export to Claude
- [ ] Export to OpenClaw
- [ ] Export to Ollama
- [ ] Import from ChatGPT (API-based)
- [ ] Import from OpenClaw workspace

### Phase 4: Skills (Weeks 17-20)
- [ ] Skill Manager
- [ ] Skill loading and activation
- [ ] Skill configuration
- [ ] CLI: `skills list`, `skills add`, `skills remove`

### Phase 5: Registry (Weeks 21-24)
- [ ] Registry server (Go)
- [ ] Registry API
- [ ] CLI: `registry push`, `registry pull`, `registry search`
- [ ] Web UI for browsing
- [ ] GitHub OAuth

### Phase 6: Polish (Weeks 25-28)
- [ ] Web UI (React)
- [ ] SDK (TypeScript)
- [ ] SDK (Python)
- [ ] Documentation site
- [ ] Templates (senior-dev, writer, researcher, etc.)

### Phase 7: Community (Ongoing)
- [ ] Open RFC process for spec changes
- [ ] Contributor guide
- [ ] Monthly community calls
- [ ] Platform adapter bounties

---

## 13. Contributing

### 13.1 Getting Started

```bash
# Clone
git clone https://github.com/mturac/agentvm.git
cd agentvm

# Build
cargo build

# Test
cargo test

# Run CLI
cargo run -- init
```

### 13.2 Contribution Areas

| Area | Difficulty | Description |
|------|-----------|-------------|
| **Platform Adapters** | 🟢 Easy | New LLM provider adapter |
| **Exporters** | 🟢 Easy | Export to new platform |
| **Importers** | 🟢 Easy | Import from new platform |
| **Templates** | 🟢 Easy | Pre-built agent templates |
| **Test Cases** | 🟢 Easy | Spec compliance tests |
| **Documentation** | 🟢 Easy | Guides, tutorials, examples |
| **Memory Engine** | 🟡 Medium | Storage, retrieval, consolidation |
| **Skill System** | 🟡 Medium | Skill loading, activation |
| **Web UI** | 🟡 Medium | React frontend |
| **CLI** | 🟡 Medium | Rust CLI features |
| **Core Spec** | 🔴 Hard | Spec design, breaking changes |
| **Registry** | 🔴 Hard | Server, auth, storage |

### 13.3 Good First Issues

Look for issues tagged `good-first-issue`:

1. **Add a platform exporter** — Export agent to a new platform (e.g., Gemini, Mistral)
2. **Write a template** — Create a pre-built agent template for a use case
3. **Add test cases** — Write spec compliance tests
4. **Improve docs** — Write guides, fix typos, add examples
5. **Memory search** — Implement a new search algorithm

### 13.4 RFC Process

Major spec changes go through an RFC (Request for Comments):

1. Create an RFC in `rfcs/` directory
2. Open a PR with the RFC
3. Community discussion (minimum 2 weeks)
4. Core team vote
5. If approved, implement

### 13.5 Code Style

- **Rust:** `rustfmt` + `clippy`
- **TypeScript:** `prettier` + `eslint`
- **Python:** `black` + `ruff`
- **Commit messages:** Conventional Commits (`feat:`, `fix:`, `docs:`)

---

## 14. FAQ

### Q: Isn't this just a vector database for agent memory?
**A:** No. A vector database stores and retrieves text. AgentVM packages an agent's *complete cognitive state* — personality, behavior patterns, learned skills, social understanding, tool preferences, and memory. A vector database is one component of the memory system.

### Q: Why not just use ChatGPT's built-in memory?
**A:** Because it's locked to OpenAI. You can't take your ChatGPT memory and use it in Claude, or in a local model, or in your own agent framework. AgentVM makes memory platform-agnostic.

### Q: How is this different from A2A (Agent-to-Agent) protocol?
**A:** A2A is about agents *communicating with each other*. AgentVM is about *packaging and moving a single agent* between platforms. They solve different problems and are complementary.

### Q: How is this different from MCP (Model Context Protocol)?
**A:** MCP is about connecting agents to *tools and data sources*. AgentVM is about the agent itself — its identity, memory, and learned behavior. An agent can use MCP tools regardless of where it's running.

### Q: Can I use this with my existing agent framework (LangChain, CrewAI, etc.)?
**A:** Yes. AgentVM sits *above* frameworks. You export an agent image to a format your framework understands, or use the runtime adapters to interact with the framework's API.

### Q: Is my data safe?
**A:** Agent images can be encrypted at rest. Sensitive fields (memory, social contacts) are encrypted by default. The registry supports private images. No telemetry is collected.

### Q: What about model-specific behaviors?
**A:** Some behaviors are model-specific (e.g., Claude's longer context window, GPT's image generation). The spec handles this through platform overrides and capability detection. The core identity and memory are always preserved.

### Q: Can I sell my agent image?
**A:** Yes. The registry supports paid images (via Stripe integration). You set the price, we handle distribution. License is up to you.

---

## Appendix A: Example Agent Images

### Senior Developer Assistant

```yaml
apiVersion: agentvm/v1
kind: AgentImage
metadata:
  name: "senior-dev"
  version: "1.0.0"
  description: "Template for a senior developer assistant"

identity:
  persona: |
    Experienced developer. Practical, efficient, security-conscious.
    Writes clean code. Explains decisions. Pushes back on bad ideas.

skills:
  builtin:
    - id: "code-review"
    - id: "architecture"
    - id: "debugging"

runtime:
  preferredModels:
    - provider: "anthropic"
      model: "claude-sonnet-4"
```

### Creative Writer

```yaml
apiVersion: agentvm/v1
kind: AgentImage
metadata:
  name: "creative-writer"
  version: "1.0.0"
  description: "Template for a creative writing assistant"

identity:
  persona: |
    Passionate storyteller. Knows narrative structure, character development,
    and dialogue. Adapts style to genre. Gives honest feedback.

skills:
  builtin:
    - id: "story-structure"
    - id: "character-development"
    - id: "dialogue-polish"

runtime:
  preferredModels:
    - provider: "anthropic"
      model: "claude-sonnet-4"
  performance:
    temperature: 0.9
```

### Research Assistant

```yaml
apiVersion: agentvm/v1
kind: AgentImage
metadata:
  name: "research-assistant"
  version: "1.0.0"
  description: "Template for an academic research assistant"

identity:
  persona: |
    Meticulous researcher. Values evidence over opinion.
    Cites sources. Identifies biases. Synthesizes complex information.

skills:
  builtin:
    - id: "literature-review"
    - id: "data-analysis"
    - id: "citation-management"

tools:
  preferred:
    webSearch:
      provider: "mimo-web-search"
    knowledge:
      provider: "memory_search"

runtime:
  preferredModels:
    - provider: "anthropic"
      model: "claude-sonnet-4"
```

---

## Appendix B: Glossary

| Term | Definition |
|------|-----------|
| **Agent Image** | A packaged cognitive state of an AI agent |
| **Episodic Memory** | Records of past events and conversations |
| **Semantic Memory** | Structured knowledge and facts |
| **Procedural Memory** | Learned skills and behavioral patterns |
| **Social Memory** | Knowledge about people and relationships |
| **Platform Adapter** | Code that translates between AgentVM and a specific platform |
| **Runtime** | The execution environment for an agent image |
| **Registry** | A server for storing and sharing agent images |
| **Consolidation** | The process of compressing and organizing memories |
| **Cognitive Portability** | The ability to move an agent's complete mental state between platforms |

---

*Spec v0.1.0 — Draft. Subject to change based on community feedback.*

*Join the discussion: github.com/mturac/agentvm/discussions*
