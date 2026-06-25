import { BaseAdapter } from "./base-adapter.js";
import type { AgentImage, PlatformCapabilities } from "./types.js";

export type OpenAIConfig = {
  name: string;
  customInstructions: string;
  knowledgeBase: string;
  gptConfig: {
    description?: string;
    tags: string[];
  };
};

export class OpenAIAdapter extends BaseAdapter<OpenAIConfig> {
  readonly platform = "openai";

  toPlatformFormat(image: AgentImage): OpenAIConfig {
    return {
      name: this.displayName(image),
      customInstructions: this.persona(image),
      knowledgeBase: this.knowledgeSummary(image),
      gptConfig: {
        description: image.metadata.description,
        tags: image.metadata.tags ?? [],
      },
    };
  }

  fromPlatformFormat(config: OpenAIConfig): AgentImage {
    return this.baseImage(config.name, config.customInstructions);
  }

  capabilities(): PlatformCapabilities {
    return {
      supportsImages: true,
      supportsAudio: true,
      supportsToolUse: true,
      supportsStreaming: true,
      supportsSystemPrompt: true,
      maxContextWindow: 128000,
      maxOutputTokens: 16384,
    };
  }
}
