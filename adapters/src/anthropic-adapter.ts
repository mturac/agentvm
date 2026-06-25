import { BaseAdapter } from "./base-adapter.js";
import type { AgentImage, PlatformCapabilities } from "./types.js";

export type AnthropicConfig = {
  projectName: string;
  projectInstructions: string;
  projectKnowledge: string;
  skills: Array<{ id: string; enabled: boolean }>;
};

export class AnthropicAdapter extends BaseAdapter<AnthropicConfig> {
  readonly platform = "anthropic";

  toPlatformFormat(image: AgentImage): AnthropicConfig {
    return {
      projectName: this.displayName(image),
      projectInstructions: this.persona(image),
      projectKnowledge: this.knowledgeSummary(image),
      skills:
        image.skills?.builtin?.map((skill) => ({
          id: skill.id,
          enabled: skill.enabled ?? true,
        })) ?? [],
    };
  }

  fromPlatformFormat(config: AnthropicConfig): AgentImage {
    return this.baseImage(config.projectName, config.projectInstructions);
  }

  capabilities(): PlatformCapabilities {
    return {
      supportsImages: true,
      supportsAudio: false,
      supportsToolUse: true,
      supportsStreaming: true,
      supportsSystemPrompt: true,
      maxContextWindow: 200000,
      maxOutputTokens: 8192,
    };
  }
}
