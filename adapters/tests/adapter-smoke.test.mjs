import assert from "node:assert/strict";
import {
  AnthropicAdapter,
  GeminiAdapter,
  OllamaAdapter,
  OpenAIAdapter,
  OpenClawAdapter,
  createDefaultAdapters,
} from "../dist/index.js";

const image = {
  apiVersion: "agentvm/v1",
  kind: "AgentImage",
  metadata: {
    name: "portable-dev",
    version: "1.0.0",
    displayName: "Portable Dev",
    description: "A developer agent that should survive platform migration.",
    tags: ["coding", "portable"],
  },
  identity: {
    name: "Portable Dev",
    persona: "A practical senior developer with durable memory.",
  },
  skills: {
    builtin: [
      { id: "code-review", version: "1.0.0", enabled: true },
      { id: "debugging", version: "1.0.0", enabled: true },
    ],
  },
  runtime: {
    preferredModels: [
      { provider: "local", model: "llama-3.3-70b", priority: 1 },
      { provider: "gemini", model: "gemini-1.5-pro", priority: 2 },
    ],
  },
};

const openai = new OpenAIAdapter().toPlatformFormat(image);
assert.equal(openai.name, "Portable Dev");
assert.match(openai.customInstructions, /practical senior developer/);
assert.match(openai.knowledgeBase, /code-review/);

const anthropic = new AnthropicAdapter().toPlatformFormat(image);
assert.equal(anthropic.projectName, "Portable Dev");
assert.equal(anthropic.skills.length, 2);

const geminiAdapter = new GeminiAdapter();
const gemini = geminiAdapter.toPlatformFormat(image);
assert.equal(gemini.gemName, "Portable Dev");
assert.match(gemini.instructions, /durable memory/);
assert.match(gemini.knowledgeBundle, /gemini\/gemini-1.5-pro/);
assert.equal(gemini.gemConfig.preferredModel, "gemini-1.5-pro");
const geminiRoundTrip = geminiAdapter.fromPlatformFormat(gemini);
assert.equal(geminiRoundTrip.metadata.name, "portable-dev");
assert.equal(geminiRoundTrip.runtime.preferredModels[0].provider, "gemini");
assert.equal(geminiRoundTrip.runtime.preferredModels[0].model, "gemini-1.5-pro");
assert.equal(geminiAdapter.capabilities().supportsToolUse, true);

const ollama = new OllamaAdapter().toPlatformFormat(image);
assert.match(ollama.modelfile, /FROM llama-3.3-70b/);
assert.match(ollama.systemPrompt, /durable memory/);

const openclaw = new OpenClawAdapter().toPlatformFormat(image);
assert.match(openclaw["SOUL.md"], /durable memory/);
assert.deepEqual(openclaw.skills, ["code-review", "debugging"]);

const adapters = createDefaultAdapters();
assert.deepEqual(
  adapters.map((adapter) => adapter.platform).sort(),
  ["anthropic", "gemini", "ollama", "openai", "openclaw"].sort(),
);

console.log("adapter smoke passed");
