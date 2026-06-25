import { BaseAdapter } from "./base-adapter.js";
import type { AgentImage, PlatformCapabilities } from "./types.js";

export type OllamaConfig = {
  modelfile: string;
  systemPrompt: string;
  context: {
    imageName: string;
    preferredModel?: string;
  };
};

export class OllamaAdapter extends BaseAdapter<OllamaConfig> {
  readonly platform = "ollama";

  toPlatformFormat(image: AgentImage): OllamaConfig {
    const preferred = image.runtime?.preferredModels?.find((model) => model.provider === "local");
    const systemPrompt = this.persona(image);
    return {
      modelfile: `FROM ${preferred?.model ?? "llama3.2"}\n\nSYSTEM \"\"\"\n${escapeTripleQuotes(systemPrompt)}\n\"\"\"\n`,
      systemPrompt,
      context: {
        imageName: image.metadata.name,
        preferredModel: preferred?.model,
      },
    };
  }

  fromPlatformFormat(config: OllamaConfig): AgentImage {
    return this.baseImage(config.context.imageName, config.systemPrompt);
  }

  capabilities(): PlatformCapabilities {
    return {
      supportsImages: false,
      supportsAudio: false,
      supportsToolUse: false,
      supportsStreaming: true,
      supportsSystemPrompt: true,
      maxContextWindow: 128000,
      maxOutputTokens: 4096,
    };
  }
}

function escapeTripleQuotes(value: string): string {
  return value.replace(/"""/g, '\\"\\"\\"');
}
