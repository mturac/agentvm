export type {
  AgentImage,
  MemoryDelta,
  PlatformAdapter,
  PlatformCapabilities,
  TurnContext,
} from "./types.js";
export { BaseAdapter } from "./base-adapter.js";
export { AnthropicAdapter, type AnthropicConfig } from "./anthropic-adapter.js";
export { GeminiAdapter, type GeminiConfig } from "./gemini-adapter.js";
export { OllamaAdapter, type OllamaConfig } from "./ollama-adapter.js";
export { OpenAIAdapter, type OpenAIConfig } from "./openai-adapter.js";
export { OpenClawAdapter, type OpenClawConfig } from "./openclaw-adapter.js";

import { AnthropicAdapter } from "./anthropic-adapter.js";
import { GeminiAdapter } from "./gemini-adapter.js";
import { OllamaAdapter } from "./ollama-adapter.js";
import { OpenAIAdapter } from "./openai-adapter.js";
import { OpenClawAdapter } from "./openclaw-adapter.js";

export function createDefaultAdapters() {
  return [
    new OpenAIAdapter(),
    new AnthropicAdapter(),
    new GeminiAdapter(),
    new OllamaAdapter(),
    new OpenClawAdapter(),
  ];
}
