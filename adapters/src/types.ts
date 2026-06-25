export type AgentImage = {
  apiVersion: "agentvm/v1";
  kind: "AgentImage";
  metadata: {
    name: string;
    version: string;
    displayName?: string;
    description?: string;
    tags?: string[];
  };
  identity?: {
    name?: string;
    persona?: string;
    tone?: Record<string, string>;
    languages?: Array<{ code: string; proficiency?: string }>;
    behaviors?: Record<string, string>;
  };
  memory?: Record<string, unknown>;
  skills?: {
    builtin?: Array<{ id: string; version?: string; enabled?: boolean }>;
  };
  tools?: {
    denied?: string[];
    security?: Record<string, unknown>;
  };
  prompts?: Record<string, unknown>;
  runtime?: {
    preferredModels?: Array<{ provider: string; model: string; priority?: number }>;
  };
  export?: Record<string, unknown>;
};

export type PlatformCapabilities = {
  supportsImages: boolean;
  supportsAudio: boolean;
  supportsToolUse: boolean;
  supportsStreaming: boolean;
  supportsSystemPrompt: boolean;
  maxContextWindow: number;
  maxOutputTokens: number;
};

export type TurnContext = {
  sessionId?: string;
  platform?: string;
  memories?: string[];
};

export type MemoryDelta = {
  semantic: Array<{ key: string; value: string; confidence: number }>;
  episodic: Array<{ summary: string; importance: number }>;
};

export interface PlatformAdapter<PlatformConfig = unknown> {
  readonly platform: string;
  toPlatformFormat(image: AgentImage): PlatformConfig;
  fromPlatformFormat(config: PlatformConfig): AgentImage;
  injectMemory(memory: string, context?: TurnContext): string;
  extractMemories(conversation: string): MemoryDelta;
  capabilities(): PlatformCapabilities;
}
