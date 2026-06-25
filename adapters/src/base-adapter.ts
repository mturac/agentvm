import type { AgentImage, MemoryDelta, PlatformCapabilities, TurnContext } from "./types.js";

export abstract class BaseAdapter<PlatformConfig> {
  abstract readonly platform: string;
  abstract toPlatformFormat(image: AgentImage): PlatformConfig;
  abstract fromPlatformFormat(config: PlatformConfig): AgentImage;
  abstract capabilities(): PlatformCapabilities;

  injectMemory(memory: string, context: TurnContext = {}): string {
    const header = context.sessionId
      ? `AgentVM portable memory for session ${context.sessionId}`
      : "AgentVM portable memory";
    return `${header}\n\n${memory.trim()}`;
  }

  extractMemories(conversation: string): MemoryDelta {
    const lines = conversation
      .split("\n")
      .map((line) => line.trim())
      .filter(Boolean);

    return {
      semantic: lines
        .filter((line) => line.toLowerCase().startsWith("remember:"))
        .map((line, index) => ({
          key: `memory-${index + 1}`,
          value: line.replace(/^remember:\s*/i, ""),
          confidence: 0.8,
        })),
      episodic: lines.slice(0, 3).map((line) => ({
        summary: line,
        importance: 0.5,
      })),
    };
  }

  protected baseImage(name: string, persona: string): AgentImage {
    return {
      apiVersion: "agentvm/v1",
      kind: "AgentImage",
      metadata: {
        name: slugify(name),
        version: "1.0.0",
        displayName: name,
      },
      identity: {
        name,
        persona,
      },
    };
  }

  protected persona(image: AgentImage): string {
    return image.identity?.persona ?? "A helpful portable AI assistant.";
  }

  protected displayName(image: AgentImage): string {
    return image.metadata.displayName ?? image.identity?.name ?? image.metadata.name;
  }

  protected knowledgeSummary(image: AgentImage): string {
    const skills = image.skills?.builtin?.map((skill) => skill.id).join(", ") || "none";
    const models =
      image.runtime?.preferredModels
        ?.map((model) => `${model.provider}/${model.model}`)
        .join(", ") || "none";
    return [
      `AgentVM image: ${image.metadata.name}@${image.metadata.version}`,
      image.metadata.description ? `Description: ${image.metadata.description}` : undefined,
      `Skills: ${skills}`,
      `Preferred models: ${models}`,
    ]
      .filter(Boolean)
      .join("\n");
  }
}

export function slugify(value: string): string {
  const slug = value
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-|-$/g, "");
  return slug || "agent";
}
