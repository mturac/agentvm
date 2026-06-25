import { BaseAdapter } from "./base-adapter.js";
import type { AgentImage, PlatformCapabilities } from "./types.js";

export type GeminiConfig = {
  gemName: string;
  instructions: string;
  knowledgeBundle: string;
  gemConfig: {
    description?: string;
    tags: string[];
    preferredModel?: string;
  };
};

export class GeminiAdapter extends BaseAdapter<GeminiConfig> {
  readonly platform = "gemini";

  toPlatformFormat(image: AgentImage): GeminiConfig {
    const preferred = image.runtime?.preferredModels?.find((model) =>
      ["gemini", "google"].some((provider) => model.provider.toLowerCase().includes(provider)),
    );
    return {
      gemName: this.displayName(image),
      instructions: this.persona(image),
      knowledgeBundle: this.knowledgeSummary(image),
      gemConfig: {
        description: image.metadata.description,
        tags: image.metadata.tags ?? [],
        preferredModel: preferred?.model,
      },
    };
  }

  fromPlatformFormat(config: GeminiConfig): AgentImage {
    const image = this.baseImage(config.gemName, config.instructions);
    image.metadata.description = config.gemConfig.description;
    image.metadata.tags = config.gemConfig.tags;
    if (config.gemConfig.preferredModel) {
      image.runtime = {
        preferredModels: [
          {
            provider: "gemini",
            model: config.gemConfig.preferredModel,
            priority: 1,
          },
        ],
      };
    }
    return image;
  }

  capabilities(): PlatformCapabilities {
    return {
      supportsImages: true,
      supportsAudio: true,
      supportsToolUse: true,
      supportsStreaming: true,
      supportsSystemPrompt: true,
      maxContextWindow: 1000000,
      maxOutputTokens: 8192,
    };
  }
}
