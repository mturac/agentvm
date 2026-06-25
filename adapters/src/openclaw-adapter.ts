import { BaseAdapter } from "./base-adapter.js";
import type { AgentImage, PlatformCapabilities } from "./types.js";

export type OpenClawConfig = {
  "SOUL.md": string;
  "USER.md": string;
  "AGENTS.md": string;
  "MEMORY.md": string;
  skills: string[];
};

export class OpenClawAdapter extends BaseAdapter<OpenClawConfig> {
  readonly platform = "openclaw";

  toPlatformFormat(image: AgentImage): OpenClawConfig {
    return {
      "SOUL.md": this.persona(image),
      "USER.md": `# ${this.displayName(image)}\n\nExported from AgentVM image ${image.metadata.name}.\n`,
      "AGENTS.md": `# Agent Instructions\n\n${this.persona(image)}\n\nPreserve portable AgentVM memory and respect denied tools.\n`,
      "MEMORY.md": this.knowledgeSummary(image),
      skills: image.skills?.builtin?.map((skill) => skill.id) ?? [],
    };
  }

  fromPlatformFormat(config: OpenClawConfig): AgentImage {
    return this.baseImage("openclaw-agent", config["SOUL.md"]);
  }

  capabilities(): PlatformCapabilities {
    return {
      supportsImages: true,
      supportsAudio: false,
      supportsToolUse: true,
      supportsStreaming: true,
      supportsSystemPrompt: true,
      maxContextWindow: 128000,
      maxOutputTokens: 8192,
    };
  }
}
