import React from "react";
import ReactDOM from "react-dom/client";
import { dump, load } from "js-yaml";
import {
  Archive,
  Brain,
  CheckCircle2,
  ChevronRight,
  CircleAlert,
  Code2,
  FileDown,
  FileInput,
  GitCompare,
  HardDriveDownload,
  PackageCheck,
  Plus,
  ShieldCheck,
  Trash2,
  UploadCloud,
} from "lucide-react";
import "./styles.css";

type AgentImage = {
  apiVersion?: string;
  kind?: string;
  metadata?: {
    name?: string;
    version?: string;
    displayName?: string;
    description?: string;
    tags?: string[];
  };
  identity?: {
    name?: string;
    persona?: string;
    tone?: Record<string, string>;
    languages?: Array<{ code: string; proficiency?: string }>;
  };
  memory?: Record<string, unknown>;
  skills?: {
    builtin?: Array<{ id: string; version?: string; enabled?: boolean; path?: string }>;
    registry?: Array<{ id: string; version: string; source: string }>;
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

type Platform = {
  key: string;
  label: string;
  output: string;
  ready: boolean;
  note: string;
};

type PlatformExport = {
  platform: string;
  filename: string;
  mime: string;
  preview: string;
  content: string;
  files: PackageFiles;
  warnings: string[];
};

type RegistryImage = {
  owner: string;
  name: string;
  version: string;
  description?: string;
  tags?: string[];
  createdAt?: string;
  manifest?: AgentImage;
  files?: Record<string, string>;
};

type AgentMutator = (draft: AgentImage) => void;
type PackageFiles = Record<string, string>;
type TemplateSeed = {
  id: string;
  label: string;
  category: string;
  summary: string;
  agent: AgentImage;
  files: PackageFiles;
};
type PlatformImportResult = {
  agent: AgentImage;
  files: PackageFiles;
  source: string;
};
type StudioSnapshot = {
  version: 1;
  agent: AgentImage;
  manifest: string;
  packageFiles: PackageFiles;
  comparisonManifest: string;
  activeTab: string;
  lastImport: string;
  selectedPlatform: string;
  packageState: number;
  registryUrl: string;
  registryOwner: string;
  registryQuery: string;
  savedAt: string;
};
type DiffSectionKey = "metadata" | "identity" | "memory" | "skills" | "tools" | "runtime" | "export";

type DiffSection = {
  key: DiffSectionKey;
  label: string;
  status: "unchanged" | "changed" | "added" | "removed";
  summary: string;
};
type SecurityFinding = {
  severity: "high" | "medium";
  path: string;
  line: number;
  rule: string;
  message: string;
};

const sampleAgent: AgentImage = {
  apiVersion: "agentvm/v1",
  kind: "AgentImage",
  metadata: {
    name: "turkish-dev",
    version: "1.0.0",
    displayName: "Turkish Dev",
    description:
      "A portable coding agent with memory, personality, skills, and tool preferences.",
    tags: ["coding", "turkish", "devops"],
  },
  identity: {
    name: "Turkish Dev",
    persona:
      "Turkish-speaking senior developer. Direct, pragmatic, and focused on shipping verified work.",
    languages: [
      { code: "tr", proficiency: "native" },
      { code: "en", proficiency: "fluent" },
    ],
  },
  memory: {
    strategy: {
      consolidationFrequency: "weekly",
      forgettingPolicy: "importance-based",
      retrievalMethod: "semantic+recency",
    },
    episodic: { source: "memory/episodic.md", format: "structured-markdown" },
    semantic: { source: "memory/semantic.json", format: "key-value-store" },
    procedural: { source: "memory/procedural.yaml", format: "skill-manifest" },
    social: { source: "memory/social.yaml", format: "contact-graph" },
  },
  skills: {
    builtin: [
      { id: "code-review", version: "1.0.0", enabled: true },
      { id: "devops", version: "1.0.0", enabled: true },
    ],
  },
  tools: {
    denied: ["tts", "network:untrusted"],
    security: {
      execPolicy: "workspace-only",
      networkPolicy: "allowlist",
    },
  },
  runtime: {
    preferredModels: [
      { provider: "anthropic", model: "claude-sonnet-4", priority: 1 },
      { provider: "local", model: "llama-3.3-70b", priority: 2 },
    ],
  },
  export: {
    openai: { customInstructions: "{{identity.persona}}" },
    anthropic: { projectInstructions: "{{prompts.system}}" },
    openclaw: { soulFile: "{{prompts.system}}" },
    ollama: { modelfile: "{{identity.persona}}" },
  },
};

const memoryDefaults: Record<string, { source: string; format: string }> = {
  episodic: { source: "memory/episodic.md", format: "structured-markdown" },
  semantic: { source: "memory/semantic.json", format: "key-value-store" },
  procedural: { source: "memory/procedural.yaml", format: "skill-manifest" },
  social: { source: "memory/social.yaml", format: "contact-graph" },
};

const memoryTabPaths: Record<string, string> = {
  "Core Memory": "memory/semantic.json",
  Episodic: "memory/episodic.md",
  Knowledge: "memory/semantic.json",
};

const studioStorageKey = "agentvm.studio.workspace.v1";

const templateSeeds: TemplateSeed[] = [
  createTemplateSeed(sampleAgent, "Engineering", "Turkish delivery-focused coding agent."),
  createTemplateSeed(
    templateAgent("senior-dev", "Senior Dev", "Practical, security-conscious software engineering assistant.", ["coding", "architecture", "debugging", "deploy"], "Experienced developer, practical and security-conscious.\nPrefers scoped patches, tests, and evidence over broad rewrites.", ["code-review", "architecture", "debugging"], [{ provider: "local", model: "llama-3.3-70b", priority: 1 }], ["network:untrusted"]),
    "Engineering",
    "Scoped code review, architecture, debugging, and deployment habits.",
  ),
  createTemplateSeed(
    templateAgent("researcher", "Researcher", "Evidence-first assistant for literature review, analysis, and citation.", ["research", "analysis", "citation"], "Meticulous researcher who separates evidence from inference,\ncites sources, and values reproducible analysis.", ["literature-review", "data-analysis"], [{ provider: "openai", model: "gpt-4o", priority: 1 }]),
    "Research",
    "Literature review and citation-oriented analysis workflow.",
  ),
  createTemplateSeed(
    templateAgent("data-analyst", "Data Analyst", "Numbers-driven assistant for SQL, statistics, and visualization.", ["data", "sql", "statistics", "charts"], "Numbers-driven, visualization-focused analyst.\nChecks assumptions, explains uncertainty, and communicates with clear charts.", ["sql-expert", "chart-generation"], [{ provider: "local", model: "llama-3.3-70b", priority: 1 }]),
    "Analysis",
    "SQL, stats, uncertainty, and chart-ready reasoning.",
  ),
  createTemplateSeed(
    templateAgent("customer-support", "Customer Support", "Friendly support assistant for triage, response drafting, and escalation.", ["support", "triage", "customer-success"], "Friendly, patient, empathetic support agent.\nClarifies issues, avoids defensiveness, and escalates when unsure.", ["ticket-triage", "empathetic-responses"], [{ provider: "gemini", model: "gemini-pro", priority: 1 }]),
    "Operations",
    "Ticket triage, support tone, and escalation behavior.",
  ),
  createTemplateSeed(
    templateAgent("creative-writer", "Creative Writer", "Story-focused assistant for narrative structure, voice, and revision.", ["writing", "story", "editing"], "Passionate storyteller who understands narrative structure,\ncharacter development, dialogue, and revision.", ["story-structure", "character-development"], [{ provider: "anthropic", model: "claude-sonnet-4", priority: 1 }]),
    "Creative",
    "Story structure, character development, dialogue, and revision.",
  ),
];

function App() {
  const initialWorkspace = React.useMemo(() => loadStudioSnapshot() ?? defaultStudioSnapshot(), []);
  const [agent, setAgent] = React.useState<AgentImage>(initialWorkspace.agent);
  const [manifest, setManifest] = React.useState(initialWorkspace.manifest);
  const [packageFiles, setPackageFiles] = React.useState<PackageFiles>(initialWorkspace.packageFiles);
  const [comparisonManifest, setComparisonManifest] = React.useState(initialWorkspace.comparisonManifest);
  const [activeTab, setActiveTab] = React.useState(initialWorkspace.activeTab);
  const [dragging, setDragging] = React.useState(false);
  const [lastImport, setLastImport] = React.useState(initialWorkspace.lastImport);
  const [selectedPlatform, setSelectedPlatform] = React.useState(initialWorkspace.selectedPlatform);
  const [packageState, setPackageState] = React.useState(initialWorkspace.packageState);
  const [registryUrl, setRegistryUrl] = React.useState(initialWorkspace.registryUrl);
  const [registryOwner, setRegistryOwner] = React.useState(initialWorkspace.registryOwner);
  const [registryQuery, setRegistryQuery] = React.useState(initialWorkspace.registryQuery);
  const [registryImages, setRegistryImages] = React.useState<RegistryImage[]>([]);
  const [registryStatus, setRegistryStatus] = React.useState("Not connected");
  const [savedAt, setSavedAt] = React.useState(initialWorkspace.savedAt);

  const validation = validateAgent(agent);
  const platforms = platformReadiness(agent);
  const portability = scorePortability(agent, platforms);
  const yamlLines = manifest.split("\n");
  const comparison = React.useMemo(() => parseAgentManifest(comparisonManifest), [comparisonManifest]);
  const diffSections = React.useMemo(
    () => compareAgentImages(agent, comparison.agent),
    [agent, comparison.agent],
  );
  const safetyFindings = React.useMemo(
    () => scanPackageSecurity(manifest, packageFiles),
    [manifest, packageFiles],
  );
  const selectedPlatformInfo =
    platforms.find((platform) => platform.key === selectedPlatform) ?? platforms[0];
  const workspaceExportPlatform = platformKeyFromTab(activeTab) ?? selectedPlatform;
  const workspaceExport = buildPlatformExport(agent, workspaceExportPlatform, packageFiles);

  React.useEffect(() => {
    const snapshot = buildStudioSnapshot({
      agent,
      manifest,
      packageFiles,
      comparisonManifest,
      activeTab,
      lastImport,
      selectedPlatform,
      packageState,
      registryUrl,
      registryOwner,
      registryQuery,
    });
    saveStudioSnapshot(snapshot);
    setSavedAt(snapshot.savedAt);
  }, [
    activeTab,
    agent,
    comparisonManifest,
    lastImport,
    manifest,
    packageFiles,
    packageState,
    registryOwner,
    registryQuery,
    registryUrl,
    selectedPlatform,
  ]);

  function commitAgent(next: AgentImage) {
    setAgent(next);
    setManifest(dump(next, { lineWidth: 88 }));
  }

  function updateAgent(mutator: AgentMutator) {
    const next = cloneAgent(agent);
    mutator(next);
    commitAgent(next);
  }

  function syncManifest(next: string) {
    setManifest(next);
    try {
      const loaded = load(next);
      if (loaded && typeof loaded === "object") {
        setAgent(loaded as AgentImage);
      }
    } catch {
      // Keep the editor usable while invalid YAML is being typed.
    }
  }

  async function importFile(file: File) {
    setLastImport(file.name);
    if (file.name.endsWith(".agentvm")) {
      setManifest(`# ${file.name}
# Binary .agentvm archive detected.
# Use the CLI unpack command for archive inspection:
# agentvm unpack ${file.name} --output ./agent
`);
      return;
    }
    const text = await file.text();
    try {
      const loaded = load(text);
      if (isBrowserPackage(loaded)) {
        const next = loaded.manifest ?? loaded.agent;
        if (!next) return;
        commitAgent(next);
        setPackageFiles(extractEditablePackageFiles(loaded.files));
        setLastImport(`${file.name} / ${loaded.package?.format ?? "agentvm-browser-bundle"}`);
        setPackageState(5);
        return;
      }
    } catch {
      // Fall through to normal manifest editing; syncManifest keeps invalid text visible.
    }
    setPackageFiles(buildDefaultPackageFiles());
    syncManifest(text);
  }

  function exportManifest() {
    downloadText(`${agent.metadata?.name ?? "agent"}.yaml`, manifest, "text/yaml");
  }

  function preparePlatformExport() {
    const bundle = buildPlatformExport(agent, selectedPlatform, packageFiles);
    downloadText(bundle.filename, bundle.content, bundle.mime);
    setPackageState(5);
  }

  function repairAgent() {
    const next = repairImage(agent);
    commitAgent(next);
    setPackageState(2);
  }

  function applyTemplate(seed: TemplateSeed) {
    commitAgent(cloneAgent(seed.agent));
    setPackageFiles(cloneData(seed.files));
    setComparisonManifest(dump(buildComparisonSeed(seed.agent), { lineWidth: 88 }));
    setLastImport(`template/${seed.id}.yaml`);
    setPackageState(3);
  }

  function applyPlatformImport(result: PlatformImportResult) {
    commitAgent(result.agent);
    setPackageFiles(result.files);
    setComparisonManifest(dump(buildComparisonSeed(result.agent), { lineWidth: 88 }));
    setLastImport(result.source);
    setPackageState(3);
  }

  function packageBrain() {
    if (safetyFindings.length > 0) return;
    const next = repairImage(agent);
    const nextManifest = dump(next, { lineWidth: 88 });
    commitAgent(next);
    const bundle = buildBrowserPackage(next, nextManifest, packageFiles);
    downloadText(bundle.filename, JSON.stringify(bundle.content, null, 2), "application/json");
    setPackageState(5);
  }

  function useCurrentAsComparison() {
    setComparisonManifest(manifest);
  }

  function resetComparisonSeed() {
    setComparisonManifest(dump(buildComparisonSeed(agent), { lineWidth: 88 }));
  }

  function mergeComparisonSection(section: DiffSectionKey) {
    if (!comparison.agent) return;
    const next = cloneAgent(agent);
    const draft = next as Record<DiffSectionKey, unknown>;
    const incoming = comparison.agent as Record<DiffSectionKey, unknown>;
    if (incoming[section] === undefined) {
      delete draft[section];
    } else {
      draft[section] = cloneData(incoming[section]);
    }
    commitAgent(next);
    setPackageState(4);
  }

  function updatePackageFile(path: string, content: string) {
    setPackageFiles((current) => ({ ...current, [path]: content }));
    setPackageState(2);
  }

  function createPackageFile(path: string, content: string) {
    const normalized = normalizePackageFilePath(path);
    if (!normalized) return null;
    setPackageFiles((current) => {
      if (current[normalized] !== undefined) return current;
      return { ...current, [normalized]: content };
    });
    setPackageState(2);
    return normalized;
  }

  function deletePackageFile(path: string) {
    setPackageFiles((current) => {
      if (current[path] === undefined || Object.keys(current).length <= 1) return current;
      const next = { ...current };
      delete next[path];
      return next;
    });
    setPackageState(2);
  }

  function resetMemoryFiles() {
    setPackageFiles(buildDefaultPackageFiles());
    setPackageState(2);
  }

  function resetWorkspace() {
    const snapshot = defaultStudioSnapshot();
    clearStudioSnapshot();
    setAgent(snapshot.agent);
    setManifest(snapshot.manifest);
    setPackageFiles(snapshot.packageFiles);
    setComparisonManifest(snapshot.comparisonManifest);
    setActiveTab(snapshot.activeTab);
    setLastImport(snapshot.lastImport);
    setSelectedPlatform(snapshot.selectedPlatform);
    setPackageState(snapshot.packageState);
    setRegistryUrl(snapshot.registryUrl);
    setRegistryOwner(snapshot.registryOwner);
    setRegistryQuery(snapshot.registryQuery);
    setRegistryImages([]);
    setRegistryStatus("Local workspace reset");
    setSavedAt(snapshot.savedAt);
  }

  async function loadRegistryImages(query = registryQuery) {
    setRegistryStatus("Loading registry...");
    try {
      const suffix = query.trim() ? `?q=${encodeURIComponent(query.trim())}` : "";
      const response = await fetch(`${registryUrl.replace(/\/$/, "")}/v1/images${suffix}`);
      if (!response.ok) throw new Error(`HTTP ${response.status}`);
      const data = (await response.json()) as { images?: RegistryImage[] };
      setRegistryImages(data.images ?? []);
      setRegistryStatus(`${data.images?.length ?? 0} image(s) loaded`);
    } catch (error) {
      setRegistryStatus(error instanceof Error ? error.message : "Registry request failed");
    }
  }

  async function publishToRegistry() {
    if (safetyFindings.length > 0) {
      setRegistryStatus("Safety scan must pass before publishing");
      return;
    }
    setRegistryStatus("Publishing current image...");
    try {
      const bundle = buildBrowserPackage(agent, manifest, packageFiles);
      const response = await fetch(`${registryUrl.replace(/\/$/, "")}/v1/images`, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({
          owner: registryOwner.trim() || "local",
          name: agent.metadata?.name ?? "agent",
          version: agent.metadata?.version ?? "1.0.0",
          description: agent.metadata?.description ?? agent.identity?.persona ?? "",
          tags: agent.metadata?.tags ?? [],
          private: false,
          manifest: bundle.content.manifest,
          files: bundle.content.files,
        }),
      });
      if (!response.ok) throw new Error(`HTTP ${response.status}`);
      const published = (await response.json()) as RegistryImage;
      await loadRegistryImages(registryQuery);
      setRegistryStatus(`Published ${published.owner}/${published.name}:${published.version}`);
    } catch (error) {
      setRegistryStatus(error instanceof Error ? error.message : "Registry publish failed");
    }
  }

  async function loadRegistryImage(image: RegistryImage) {
    setRegistryStatus(`Loading ${image.owner}/${image.name}:${image.version}...`);
    try {
      const baseUrl = registryUrl.replace(/\/$/, "");
      const owner = encodeURIComponent(image.owner);
      const name = encodeURIComponent(image.name);
      const version = encodeURIComponent(image.version);
      const response = await fetch(`${baseUrl}/v1/images/${owner}/${name}/${version}`);
      if (!response.ok) throw new Error(`HTTP ${response.status}`);
      const pulled = (await response.json()) as RegistryImage;
      if (!pulled.manifest) throw new Error("Registry image has no embedded manifest");
      commitAgent(pulled.manifest);
      setPackageFiles(extractEditablePackageFiles(pulled.files));
      setLastImport(`registry://${pulled.owner}/${pulled.name}:${pulled.version}`);
      setPackageState(5);
      setRegistryStatus(`Loaded ${pulled.owner}/${pulled.name}:${pulled.version}`);
    } catch (error) {
      setRegistryStatus(error instanceof Error ? error.message : "Registry load failed");
    }
  }

  return (
    <main
      className="studio"
      onDragOver={(event) => {
        event.preventDefault();
        setDragging(true);
      }}
      onDragLeave={() => setDragging(false)}
      onDrop={(event) => {
        event.preventDefault();
        setDragging(false);
        const file = event.dataTransfer.files[0];
        if (file) {
          void importFile(file);
        }
      }}
    >
      {dragging && (
        <div className="dropOverlay">
          <UploadCloud />
          <strong>Drop an Agent Image</strong>
          <span>agent.yaml, .yml, .json, or .agentvm archive</span>
        </div>
      )}

      <Sidebar activeTab={activeTab} onChange={setActiveTab} />

      <section className="workspace">
        <header className="topbar">
          <div>
            <strong>AgentVM Studio</strong>
            <span>Your agent's brain, packaged and portable.</span>
          </div>
          <div className="topActions">
            <span className="saveStatus">Saved locally {formatSavedAt(savedAt)}</span>
            <button className="ghostButton" onClick={resetWorkspace}>
              Reset Workspace
            </button>
            <button className="ghostButton" onClick={() => syncManifest(dump(agent, { lineWidth: 88 }))}>
              <Code2 size={16} />
              Format YAML
            </button>
            <button className="ghostButton" onClick={exportManifest}>
              <FileDown size={16} />
              Download YAML
            </button>
            <button
              className="primaryButton"
              disabled={safetyFindings.length > 0}
              onClick={packageBrain}
              title={safetyFindings.length > 0 ? "Fix Safety Scan findings before packaging" : "Download browser AgentVM bundle"}
            >
              <PackageCheck size={17} />
              Package Brain
            </button>
          </div>
        </header>

        <div className="contentGrid">
          <section className="mainColumn">
            {activeTab === "Diff" ? (
              <DiffPanel
                currentManifest={manifest}
                comparisonManifest={comparisonManifest}
                comparisonError={comparison.error}
                diffSections={diffSections}
                onComparisonChange={setComparisonManifest}
                onUseCurrent={useCurrentAsComparison}
                onResetSeed={resetComparisonSeed}
                onMergeSection={mergeComparisonSection}
              />
            ) : activeTab === "Manifest" ? (
              <ManifestEditor
                manifest={manifest}
                yamlLines={yamlLines}
                lastImport={lastImport}
                onFormat={() => syncManifest(dump(agent, { lineWidth: 88 }))}
                onChange={syncManifest}
              />
            ) : isMemoryTab(activeTab) ? (
              <MemoryExplorer
                activeTab={activeTab}
                files={packageFiles}
                onFileChange={updatePackageFile}
                onFileCreate={createPackageFile}
                onFileDelete={deletePackageFile}
                onReset={resetMemoryFiles}
              />
            ) : isExportTab(activeTab) ? (
              <PlatformExportWorkspace
                platform={workspaceExportPlatform}
                exportPackage={workspaceExport}
                platforms={platforms}
                onPlatformChange={(platform) => {
                  setSelectedPlatform(platform);
                  setActiveTab(platformTabFromKey(platform));
                }}
              />
            ) : (
              <>
                <div className="dropCard">
                  <div className="dropIcon">
                    <UploadCloud size={24} />
                  </div>
                  <div>
                    <strong>Import an agent brain</strong>
                    <span>
                      Drop a manifest, then tune persona, memory, skills, tools, and model targets
                      without writing YAML.
                    </span>
                  </div>
                  <label className="fileButton">
                    <FileInput size={16} />
                    Choose file
                    <input
                      type="file"
                      accept=".yaml,.yml,.json,.agentvm"
                      onChange={(event) => {
                        const file = event.target.files?.[0];
                        if (file) {
                          void importFile(file);
                        }
                      }}
                    />
                  </label>
                </div>

                <PlatformImportWizard onImport={applyPlatformImport} />

                <section className="brainSummary">
                  <div>
                    <span className="eyeless">Current image</span>
                    <h1>{agent.metadata?.displayName ?? agent.metadata?.name ?? "Untitled Agent"}</h1>
                    <p>
                      {agent.metadata?.description ??
                        agent.identity?.persona ??
                        "Portable agent image."}
                    </p>
                  </div>
                  <div className="scoreBlock">
                    <span>Portability</span>
                    <strong>{portability}</strong>
                    <small>/100</small>
                  </div>
                </section>

                <TemplateGallery
                  activeTemplate={agent.metadata?.name}
                  templates={templateSeeds}
                  onApply={applyTemplate}
                />

                <BrainBuilder agent={agent} onChange={updateAgent} />

                <RegistryPanel
                  agent={agent}
                  safetyFindings={safetyFindings}
                  registryUrl={registryUrl}
                  registryOwner={registryOwner}
                  registryQuery={registryQuery}
                  registryImages={registryImages}
                  registryStatus={registryStatus}
                  onUrlChange={setRegistryUrl}
                  onOwnerChange={setRegistryOwner}
                  onQueryChange={setRegistryQuery}
                  onRefresh={() => void loadRegistryImages()}
                  onPublish={() => void publishToRegistry()}
                  onSearch={() => void loadRegistryImages(registryQuery)}
                  onLoad={(image) => void loadRegistryImage(image)}
                />

                <Pipeline packageState={packageState} />
              </>
            )}
          </section>

          <aside className="inspector">
            <section className="inspectorCard">
              <h2>Validation</h2>
              <StatusLine
                ok={validation.errors.length === 0}
                label={
                  validation.errors.length === 0
                    ? "All required checks passed"
                    : `${validation.errors.length} issue(s) found`
                }
              />
              {validation.errors.map((error) => (
                <p className="errorText" key={error}>{error}</p>
              ))}
              {validation.warnings.map((warning) => (
                <p className="warning" key={warning}>{warning}</p>
              ))}
              {(validation.errors.length > 0 || validation.warnings.length > 0) && (
                <button className="ghostButton wide actionFix" onClick={repairAgent}>
                  <CheckCircle2 size={16} />
                  Fix image basics
                </button>
              )}
            </section>

            <SafetyScanCard findings={safetyFindings} />

            <section className="inspectorCard">
              <h2>Brain Components</h2>
              <Metric label="Memory sources" value={countMemory(agent)} />
              <Metric label="Skills" value={countSkills(agent)} />
              <Metric label="Denied tools" value={agent.tools?.denied?.length ?? 0} />
              <Metric label="Model targets" value={agent.runtime?.preferredModels?.length ?? 0} />
            </section>

            <section className="inspectorCard">
              <div className="panelHeader compact">
                <h2>Platform Readiness</h2>
                <GitCompare size={16} />
              </div>
              <div className="platformList">
                {platforms.map((platform) => (
                  <button
                    key={platform.key}
                    className={platform.key === selectedPlatform ? "platform active" : "platform"}
                    onClick={() => setSelectedPlatform(platform.key)}
                  >
                    <span>{platform.label}</span>
                    <small>{platform.ready ? "Ready" : platform.note}</small>
                    {platform.ready ? <CheckCircle2 size={16} /> : <CircleAlert size={16} />}
                  </button>
                ))}
              </div>
            </section>

            <section className="inspectorCard exportCard">
              <h2>{selectedPlatformInfo.label} Export</h2>
              <p>{selectedPlatformInfo.output}</p>
              <pre>{buildPlatformExport(agent, selectedPlatform, packageFiles).preview}</pre>
              <button className="primaryButton wide" onClick={preparePlatformExport}>
                <HardDriveDownload size={17} />
                Prepare Export
              </button>
            </section>
          </aside>
        </div>
      </section>
    </main>
  );
}

function TemplateGallery({
  activeTemplate,
  templates,
  onApply,
}: {
  activeTemplate?: string;
  templates: TemplateSeed[];
  onApply: (template: TemplateSeed) => void;
}) {
  return (
    <section className="templatePanel">
      <div className="panelHeader">
        <div>
          <h2>Starter Images</h2>
          <span>Pick a complete agent brain, then edit it visually or export it right away.</span>
        </div>
        <span className="pill">{templates.length} template(s)</span>
      </div>
      <div className="templateGrid">
        {templates.map((template) => {
          const selected = template.agent.metadata?.name === activeTemplate;
          return (
            <button
              className={selected ? "templateCard active" : "templateCard"}
              key={template.id}
              onClick={() => onApply(template)}
            >
              <span>{template.category}</span>
              <strong>{template.label}</strong>
              <small>{template.summary}</small>
              <em>{template.agent.metadata?.tags?.join(", ")}</em>
            </button>
          );
        })}
      </div>
    </section>
  );
}

function PlatformImportWizard({ onImport }: { onImport: (result: PlatformImportResult) => void }) {
  const [platform, setPlatform] = React.useState("chatgpt");
  const [displayName, setDisplayName] = React.useState("Migrated Agent");
  const [instructions, setInstructions] = React.useState(platformImportExample("chatgpt"));
  const [memoryNotes, setMemoryNotes] = React.useState(
    "User prefers direct answers, verified work, and keeping agent context portable.",
  );
  const ready = instructions.trim().length > 0;

  function changePlatform(nextPlatform: string) {
    setPlatform(nextPlatform);
    setInstructions(platformImportExample(nextPlatform));
  }

  return (
    <section className="platformImportPanel">
      <div className="panelHeader">
        <div>
          <h2>Import From Platform</h2>
          <span>Paste what you already have in another AI tool and turn it into an AgentVM image.</span>
        </div>
        <span className="pill">No YAML required</span>
      </div>
      <div className="platformImportBody">
        <div className="platformImportControls">
          <label>
            Source platform
            <div className="platformImportTabs" role="tablist" aria-label="Import source platform">
              {["chatgpt", "claude", "gemini", "openclaw", "ollama"].map((item) => (
                <button
                  className={item === platform ? "platformTab active" : "platformTab"}
                  key={item}
                  onClick={() => changePlatform(item)}
                >
                  {platformLabel(item)}
                </button>
              ))}
            </div>
          </label>
          <label>
            Agent name
            <input value={displayName} onChange={(event) => setDisplayName(event.target.value)} />
          </label>
        </div>
        <div className="platformImportEditors">
          <label>
            Existing instructions, SOUL.md, project prompt, or Modelfile SYSTEM
            <textarea
              spellCheck={false}
              value={instructions}
              onChange={(event) => setInstructions(event.target.value)}
            />
          </label>
          <label>
            Memory notes to preserve
            <textarea
              spellCheck={false}
              value={memoryNotes}
              onChange={(event) => setMemoryNotes(event.target.value)}
            />
          </label>
        </div>
        <div className="platformImportFooter">
          <span>{ready ? "Ready to create an AgentVM image." : "Paste platform instructions first."}</span>
          <button
            className="primaryButton"
            disabled={!ready}
            onClick={() => onImport(importPlatformText(platform, displayName, instructions, memoryNotes))}
          >
            <PackageCheck size={17} />
            Create AgentVM Image
          </button>
        </div>
      </div>
    </section>
  );
}

function MemoryExplorer({
  activeTab,
  files,
  onFileChange,
  onFileCreate,
  onFileDelete,
  onReset,
}: {
  activeTab: string;
  files: PackageFiles;
  onFileChange: (path: string, content: string) => void;
  onFileCreate: (path: string, content: string) => string | null;
  onFileDelete: (path: string) => void;
  onReset: () => void;
}) {
  const [selectedPath, setSelectedPath] = React.useState(memoryTabPaths[activeTab] ?? "memory/semantic.json");
  const [query, setQuery] = React.useState("");
  const [newPath, setNewPath] = React.useState("memory/custom-context.md");
  const [newKind, setNewKind] = React.useState("memory");
  const [createError, setCreateError] = React.useState("");
  const fileEntries = React.useMemo(
    () => Object.entries(files).sort(([left], [right]) => left.localeCompare(right)),
    [files],
  );
  const filteredEntries = fileEntries.filter(([path, content]) => {
    const needle = query.trim().toLowerCase();
    if (!needle) return true;
    return path.toLowerCase().includes(needle) || content.toLowerCase().includes(needle);
  });
  const selectedContent = files[selectedPath] ?? "";
  const canDeleteSelected = fileEntries.length > 1 && files[selectedPath] !== undefined;

  React.useEffect(() => {
    const tabPath = memoryTabPaths[activeTab];
    if (tabPath && files[tabPath] !== undefined) {
      setSelectedPath(tabPath);
      return;
    }
    if (files[selectedPath] === undefined && fileEntries[0]) {
      setSelectedPath(fileEntries[0][0]);
    }
  }, [activeTab, fileEntries, files, selectedPath]);

  function createFile() {
    const normalized = normalizePackageFilePath(newPath);
    if (!normalized) {
      setCreateError("Use a relative package path such as memory/customer-notes.md.");
      return;
    }
    if (files[normalized] !== undefined) {
      setCreateError(`${normalized} already exists.`);
      setSelectedPath(normalized);
      return;
    }
    const createdPath = onFileCreate(normalized, defaultPackageFileContent(normalized, newKind));
    if (!createdPath) {
      setCreateError("Could not create that package file.");
      return;
    }
    setCreateError("");
    setSelectedPath(createdPath);
    setQuery("");
  }

  function deleteSelectedFile() {
    if (!canDeleteSelected) return;
    const currentIndex = fileEntries.findIndex(([path]) => path === selectedPath);
    const fallback = fileEntries[currentIndex + 1]?.[0] ?? fileEntries[currentIndex - 1]?.[0] ?? fileEntries[0]?.[0];
    onFileDelete(selectedPath);
    if (fallback) setSelectedPath(fallback);
  }

  return (
    <section className="memoryPanel">
      <div className="panelHeader">
        <div>
          <h2>Memory Explorer</h2>
          <span>Edit the files that will be packaged into this AgentVM brain.</span>
        </div>
        <span className="pill">{fileEntries.length} file(s)</span>
      </div>
      <div className="memoryToolbar">
        <label>
          Search memory
          <input
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            placeholder="preference, fact, workflow..."
          />
        </label>
        <button className="ghostButton" onClick={onReset}>Reset memory files</button>
        <button
          className="ghostButton"
          onClick={() => downloadText(selectedPath.split("/").pop() ?? "memory.txt", selectedContent, "text/plain")}
        >
          <FileDown size={16} />
          Download file
        </button>
      </div>
      <div className="memoryCreateBar">
        <label>
          New package file
          <input
            value={newPath}
            onChange={(event) => {
              setNewPath(event.target.value);
              setCreateError("");
            }}
            placeholder="memory/customer-notes.md"
          />
        </label>
        <label>
          Template
          <select value={newKind} onChange={(event) => setNewKind(event.target.value)}>
            <option value="memory">Memory note</option>
            <option value="prompt">Prompt</option>
            <option value="skill">Skill</option>
            <option value="json">JSON</option>
          </select>
        </label>
        <button className="primaryButton" onClick={createFile}>
          <Plus size={16} />
          Add file
        </button>
        <button
          className="ghostButton"
          disabled={!canDeleteSelected}
          onClick={deleteSelectedFile}
          title={canDeleteSelected ? `Remove ${selectedPath} from this package` : "Keep at least one package file"}
        >
          <Trash2 size={16} />
          Delete selected
        </button>
        {createError && <p className="errorText memoryCreateError">{createError}</p>}
      </div>
      <div className="memoryWorkspace">
        <div className="memoryFileList">
          {filteredEntries.map(([path, content]) => (
            <button
              className={path === selectedPath ? "memoryFile active" : "memoryFile"}
              key={path}
              onClick={() => setSelectedPath(path)}
            >
              <strong>{path}</strong>
              <span>{summarizeMemoryFile(content)}</span>
            </button>
          ))}
          {filteredEntries.length === 0 && <p className="emptyState">No memory files match that search.</p>}
        </div>
        <label className="memoryEditor">
          {selectedPath}
          <textarea
            spellCheck={false}
            value={selectedContent}
            onChange={(event) => onFileChange(selectedPath, event.target.value)}
            aria-label="Memory file editor"
          />
        </label>
      </div>
    </section>
  );
}

function PlatformExportWorkspace({
  platform,
  exportPackage,
  platforms,
  onPlatformChange,
}: {
  platform: string;
  exportPackage: PlatformExport;
  platforms: Platform[];
  onPlatformChange: (platform: string) => void;
}) {
  const [selectedFile, setSelectedFile] = React.useState(exportPackage.filename);
  const fileEntries = React.useMemo(
    () => Object.entries(exportPackage.files).sort(([left], [right]) => left.localeCompare(right)),
    [exportPackage.files],
  );
  const selectedContent = exportPackage.files[selectedFile] ?? exportPackage.content;

  React.useEffect(() => {
    setSelectedFile(exportPackage.filename);
  }, [exportPackage.filename, platform]);

  return (
    <section className="platformExportPanel">
      <div className="panelHeader">
        <div>
          <h2>Platform Export</h2>
          <span>Preview exactly what AgentVM will hand to the target platform.</span>
        </div>
        <span className="pill">{platformLabel(platform)} bundle</span>
      </div>
      <div className="exportWorkspaceHeader">
        <div className="platformTabs" role="tablist" aria-label="Export platform">
          {platforms.map((item) => (
            <button
              className={item.key === platform ? "platformTab active" : "platformTab"}
              key={item.key}
              onClick={() => onPlatformChange(item.key)}
            >
              {item.label}
            </button>
          ))}
        </div>
        <div className="exportActions">
          <button
            className="ghostButton"
            onClick={() => downloadText(selectedFile, selectedContent, "text/plain")}
          >
            <FileDown size={16} />
            Download file
          </button>
          <button
            className="primaryButton"
            onClick={() =>
              downloadText(
                exportPackage.filename.replace(/\.[^.]+$/, ".agentvm-export.json"),
                JSON.stringify(buildExportBundle(exportPackage), null, 2),
                "application/json",
              )
            }
          >
            <PackageCheck size={17} />
            Download bundle
          </button>
        </div>
      </div>
      {exportPackage.warnings.length > 0 && (
        <div className="exportWarnings">
          {exportPackage.warnings.map((warning) => (
            <span key={warning}>{warning}</span>
          ))}
        </div>
      )}
      <div className="exportWorkspace">
        <div className="exportFileList">
          {fileEntries.map(([path, content]) => (
            <button
              className={path === selectedFile ? "exportFile active" : "exportFile"}
              key={path}
              onClick={() => setSelectedFile(path)}
            >
              <strong>{path}</strong>
              <span>{content.split("\n").length} line(s)</span>
            </button>
          ))}
        </div>
        <div className="exportPreview">
          <div className="exportPreviewHeader">
            <strong>{selectedFile}</strong>
            <span>{exportPackage.preview}</span>
          </div>
          <pre>{selectedContent}</pre>
        </div>
      </div>
    </section>
  );
}

function ManifestEditor({
  manifest,
  yamlLines,
  lastImport,
  onFormat,
  onChange,
}: {
  manifest: string;
  yamlLines: string[];
  lastImport: string;
  onFormat: () => void;
  onChange: (value: string) => void;
}) {
  return (
    <section className="editorPanel">
      <div className="panelHeader">
        <div>
          <h2>Agent Image Manifest</h2>
          <span>{lastImport}</span>
        </div>
        <button className="ghostButton" onClick={onFormat}>
          <Code2 size={16} />
          Format
        </button>
      </div>
      <div className="editorShell">
        <div className="lineNumbers">
          {yamlLines.map((_, index) => (
            <span key={index}>{index + 1}</span>
          ))}
        </div>
        <textarea
          spellCheck={false}
          value={manifest}
          onChange={(event) => onChange(event.target.value)}
          aria-label="Agent Image manifest editor"
        />
      </div>
    </section>
  );
}

function DiffPanel({
  currentManifest,
  comparisonManifest,
  comparisonError,
  diffSections,
  onComparisonChange,
  onUseCurrent,
  onResetSeed,
  onMergeSection,
}: {
  currentManifest: string;
  comparisonManifest: string;
  comparisonError?: string;
  diffSections: DiffSection[];
  onComparisonChange: (value: string) => void;
  onUseCurrent: () => void;
  onResetSeed: () => void;
  onMergeSection: (section: DiffSectionKey) => void;
}) {
  const changedCount = diffSections.filter((section) => section.status !== "unchanged").length;

  return (
    <section className="diffPanel">
      <div className="panelHeader">
        <div>
          <h2>Diff And Merge</h2>
          <span>Compare this agent image with another manifest and merge selected sections.</span>
        </div>
        <span className="pill">{changedCount} change(s)</span>
      </div>
      <div className="diffToolbar">
        <button className="ghostButton" onClick={onUseCurrent}>Use current as baseline</button>
        <button className="ghostButton" onClick={onResetSeed}>Load fork sample</button>
        <label className="fileButton compactFileButton">
          <FileInput size={16} />
          Load comparison file
          <input
            type="file"
            accept=".yaml,.yml,.json,.agentvm"
            onChange={(event) => {
              const file = event.target.files?.[0];
              if (!file) return;
              void file.text().then(onComparisonChange);
            }}
          />
        </label>
      </div>
      <div className="diffEditors">
        <label>
          Current image
          <textarea readOnly value={currentManifest} aria-label="Current Agent Image manifest" />
        </label>
        <label>
          Comparison image
          <textarea
            spellCheck={false}
            value={comparisonManifest}
            onChange={(event) => onComparisonChange(event.target.value)}
            aria-label="Comparison Agent Image manifest"
          />
        </label>
      </div>
      {comparisonError ? (
        <p className="errorText diffError">{comparisonError}</p>
      ) : (
        <div className="diffList">
          {diffSections.map((section) => (
            <div className={`diffRow ${section.status}`} key={section.key}>
              <div>
                <strong>{section.label}</strong>
                <span>{section.summary}</span>
              </div>
              <div className="diffActions">
                <small>{section.status}</small>
                {section.status !== "unchanged" && (
                  <button className="ghostButton compactButton" onClick={() => onMergeSection(section.key)}>
                    Merge
                  </button>
                )}
              </div>
            </div>
          ))}
        </div>
      )}
    </section>
  );
}

function RegistryPanel({
  agent,
  safetyFindings,
  registryUrl,
  registryOwner,
  registryQuery,
  registryImages,
  registryStatus,
  onUrlChange,
  onOwnerChange,
  onQueryChange,
  onRefresh,
  onPublish,
  onSearch,
  onLoad,
}: {
  agent: AgentImage;
  safetyFindings: SecurityFinding[];
  registryUrl: string;
  registryOwner: string;
  registryQuery: string;
  registryImages: RegistryImage[];
  registryStatus: string;
  onUrlChange: (value: string) => void;
  onOwnerChange: (value: string) => void;
  onQueryChange: (value: string) => void;
  onRefresh: () => void;
  onPublish: () => void;
  onSearch: () => void;
  onLoad: (image: RegistryImage) => void;
}) {
  const publishBlocked = safetyFindings.length > 0;
  return (
    <section className="registryPanel">
      <div className="panelHeader">
        <div>
          <h2>Local Registry</h2>
          <span>Publish and discover portable agent images from a local registry API.</span>
        </div>
        <span className="pill">{registryStatus}</span>
      </div>
      <div className="registryControls">
        <label>
          Registry URL
          <input value={registryUrl} onChange={(event) => onUrlChange(event.target.value)} />
        </label>
        <label>
          Owner
          <input value={registryOwner} onChange={(event) => onOwnerChange(event.target.value)} />
        </label>
        <label>
          Search
          <input
            value={registryQuery}
            onChange={(event) => onQueryChange(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === "Enter") onSearch();
            }}
          />
        </label>
        <div className="registryActions">
          <button className="ghostButton" onClick={onRefresh}>List</button>
          <button className="ghostButton" onClick={onSearch}>Search</button>
          <button
            className="primaryButton"
            disabled={publishBlocked}
            onClick={onPublish}
            title={publishBlocked ? "Fix Safety Scan findings before publishing" : "Publish image to registry"}
          >
            Publish {agent.metadata?.name ?? "agent"}
          </button>
        </div>
        {publishBlocked && (
          <p className="registryWarning">
            Publishing is blocked until Safety Scan has no secret-like findings.
          </p>
        )}
      </div>
      <div className="registryResults">
        {registryImages.length === 0 ? (
          <p className="emptyState">No registry images loaded.</p>
        ) : (
          registryImages.map((image) => (
            <div className="registryRow" key={`${image.owner}/${image.name}:${image.version}`}>
              <div>
                <strong>{image.owner}/{image.name}:{image.version}</strong>
                <span>{image.description || "No description"}</span>
              </div>
              <div className="registryRowActions">
                <small>{image.tags?.join(", ") || "untagged"}</small>
                <button className="ghostButton compactButton" onClick={() => onLoad(image)}>
                  <FileInput size={15} />
                  Load
                </button>
              </div>
            </div>
          ))
        )}
      </div>
    </section>
  );
}

function SafetyScanCard({ findings }: { findings: SecurityFinding[] }) {
  const visibleFindings = findings.slice(0, 5);
  return (
    <section className="inspectorCard">
      <h2>Safety Scan</h2>
      <StatusLine
        ok={findings.length === 0}
        label={
          findings.length === 0
            ? "No secret-like content found"
            : `${findings.length} secret-like finding(s)`
        }
      />
      {visibleFindings.map((finding) => (
        <p className="securityFinding" key={`${finding.path}:${finding.line}:${finding.rule}`}>
          <strong>{finding.severity}</strong> {finding.path}:{finding.line} [{finding.rule}] {finding.message}
        </p>
      ))}
      {findings.length > visibleFindings.length && (
        <p className="warning">Showing first {visibleFindings.length}; fix these and scan again.</p>
      )}
    </section>
  );
}

function BrainBuilder({ agent, onChange }: { agent: AgentImage; onChange: (mutator: AgentMutator) => void }) {
  const [skillInput, setSkillInput] = React.useState("registry://skills/github-advanced");
  const [toolInput, setToolInput] = React.useState("browser:profile");
  const [providerInput, setProviderInput] = React.useState("openai");
  const [modelInput, setModelInput] = React.useState("gpt-4.1");
  const builtinSkills = agent.skills?.builtin ?? [];
  const registrySkills = agent.skills?.registry ?? [];
  const deniedTools = agent.tools?.denied ?? [];
  const models = agent.runtime?.preferredModels ?? [];

  return (
    <section className="builderPanel">
      <div className="panelHeader">
        <div>
          <h2>Brain Builder</h2>
          <span>Visual controls update the Agent Image manifest automatically.</span>
        </div>
        <span className="pill">No YAML required</span>
      </div>

      <div className="builderGrid">
        <div className="builderSection identityEditor">
          <h3>Identity</h3>
          <label>
            Display name
            <input
              value={agent.metadata?.displayName ?? ""}
              onChange={(event) => {
                const value = event.target.value;
                onChange((draft) => {
                  ensureMetadata(draft).displayName = value;
                });
              }}
            />
          </label>
          <label>
            Image name
            <input
              value={agent.metadata?.name ?? ""}
              onChange={(event) => {
                const value = slugify(event.target.value);
                onChange((draft) => {
                  ensureMetadata(draft).name = value;
                });
              }}
            />
          </label>
          <label>
            Persona
            <textarea
              value={agent.identity?.persona ?? ""}
              onChange={(event) => {
                const value = event.target.value;
                onChange((draft) => {
                  ensureIdentity(draft).persona = value;
                });
              }}
            />
          </label>
        </div>

        <div className="builderSection">
          <h3>Memory Sources</h3>
          <div className="toggleList">
            {Object.entries(memoryDefaults).map(([key, defaults]) => (
              <label className="toggleRow" key={key}>
                <input
                  type="checkbox"
                  checked={Boolean(agent.memory?.[key])}
                  onChange={(event) => {
                    const checked = event.target.checked;
                    onChange((draft) => {
                      const memory = ensureMemory(draft);
                      if (checked) {
                        memory[key] = defaults;
                      } else {
                        delete memory[key];
                      }
                    });
                  }}
                />
                <span>
                  <strong>{key}</strong>
                  <small>{defaults.source}</small>
                </span>
              </label>
            ))}
          </div>
        </div>

        <div className="builderSection">
          <h3>Skills</h3>
          <div className="addRow">
            <input
              value={skillInput}
              onChange={(event) => setSkillInput(event.target.value)}
              placeholder="code-review or registry://skills/github-advanced"
            />
            <button
              className="iconButton"
              aria-label="Add skill"
              onClick={() => {
                const skill = parseSkillReference(skillInput);
                if (!skill) return;
                onChange((draft) => {
                  const skills = ensureSkills(draft);
                  if (skill.kind === "registry") {
                    skills.registry = [
                      ...(skills.registry ?? []).filter((entry) => entry.id !== skill.id),
                      { id: skill.id, version: skill.version, source: skill.source },
                    ];
                  } else {
                    skills.builtin = [
                      ...(skills.builtin ?? []).filter((entry) => entry.id !== skill.id),
                      { id: skill.id, version: skill.version, enabled: true },
                    ];
                  }
                });
                setSkillInput("");
              }}
            >
              <Plus size={16} />
            </button>
          </div>
          <SkillList
            builtinSkills={builtinSkills}
            registrySkills={registrySkills}
            onChange={onChange}
          />
        </div>

        <div className="builderSection">
          <h3>Tools And Models</h3>
          <div className="addRow">
            <input
              value={toolInput}
              onChange={(event) => setToolInput(event.target.value)}
              placeholder="tool to deny"
            />
            <button
              className="iconButton"
              aria-label="Deny tool"
              onClick={() => {
                const value = toolInput.trim();
                if (!value) return;
                onChange((draft) => {
                  const tools = ensureTools(draft);
                  tools.denied = Array.from(new Set([...(tools.denied ?? []), value]));
                });
                setToolInput("");
              }}
            >
              <Plus size={16} />
            </button>
          </div>
          <ChipList
            values={deniedTools}
            onRemove={(value) =>
              onChange((draft) => {
                ensureTools(draft).denied = (draft.tools?.denied ?? []).filter((tool) => tool !== value);
              })
            }
          />
          <div className="modelRow">
            <input
              value={providerInput}
              onChange={(event) => setProviderInput(event.target.value)}
              placeholder="provider"
            />
            <input
              value={modelInput}
              onChange={(event) => setModelInput(event.target.value)}
              placeholder="model"
            />
            <button
              className="iconButton"
              aria-label="Add model"
              onClick={() => {
                const provider = providerInput.trim();
                const model = modelInput.trim();
                if (!provider || !model) return;
                onChange((draft) => {
                  const runtime = ensureRuntime(draft);
                  runtime.preferredModels = [
                    ...(runtime.preferredModels ?? []),
                    { provider, model, priority: (runtime.preferredModels?.length ?? 0) + 1 },
                  ];
                });
                setModelInput("");
              }}
            >
              <Plus size={16} />
            </button>
          </div>
          <div className="itemList">
            {models.map((model, index) => (
              <div className="listItem" key={`${model.provider}-${model.model}-${index}`}>
                <span>{model.provider}/{model.model}</span>
                <button
                  aria-label={`Remove ${model.model}`}
                  onClick={() =>
                    onChange((draft) => {
                      ensureRuntime(draft).preferredModels = (draft.runtime?.preferredModels ?? []).filter(
                        (_, modelIndex) => modelIndex !== index,
                      );
                    })
                  }
                >
                  <Trash2 size={15} />
                </button>
              </div>
            ))}
          </div>
        </div>
      </div>
    </section>
  );
}

function SkillList({
  builtinSkills,
  registrySkills,
  onChange,
}: {
  builtinSkills: NonNullable<AgentImage["skills"]>["builtin"];
  registrySkills: NonNullable<AgentImage["skills"]>["registry"];
  onChange: (mutator: AgentMutator) => void;
}) {
  return (
    <div className="itemList">
      {(builtinSkills ?? []).map((skill) => (
        <div className="listItem" key={`builtin-${skill.id}`}>
          <label className="inlineCheck">
            <input
              type="checkbox"
              checked={skill.enabled ?? true}
              onChange={(event) => {
                const checked = event.target.checked;
                onChange((draft) => {
                  const match = draft.skills?.builtin?.find((entry) => entry.id === skill.id);
                  if (match) match.enabled = checked;
                });
              }}
            />
            <span>{skill.id}</span>
          </label>
          <button
            aria-label={`Remove ${skill.id}`}
            onClick={() =>
              onChange((draft) => {
                ensureSkills(draft).builtin = (draft.skills?.builtin ?? []).filter(
                  (entry) => entry.id !== skill.id,
                );
              })
            }
          >
            <Trash2 size={15} />
          </button>
        </div>
      ))}
      {(registrySkills ?? []).map((skill) => (
        <div className="listItem" key={`registry-${skill.id}`}>
          <span>{skill.id}@{skill.version}</span>
          <button
            aria-label={`Remove ${skill.id}`}
            onClick={() =>
              onChange((draft) => {
                ensureSkills(draft).registry = (draft.skills?.registry ?? []).filter(
                  (entry) => entry.id !== skill.id,
                );
              })
            }
          >
            <Trash2 size={15} />
          </button>
        </div>
      ))}
    </div>
  );
}

function ChipList({ values, onRemove }: { values: string[]; onRemove: (value: string) => void }) {
  if (!values.length) {
    return <p className="emptyState">No denied tools yet.</p>;
  }
  return (
    <div className="chipList">
      {values.map((value) => (
        <button className="chip" key={value} onClick={() => onRemove(value)}>
          {value}
          <Trash2 size={13} />
        </button>
      ))}
    </div>
  );
}

function Sidebar({ activeTab, onChange }: { activeTab: string; onChange: (tab: string) => void }) {
  const groups = [
    { title: "Brain Package", items: ["Dashboard", "Manifest", "Diff"] },
    { title: "Memory", items: ["Core Memory", "Episodic", "Knowledge"] },
    { title: "Personality", items: ["Traits", "Tone", "Rules"] },
    { title: "Skills", items: ["Skill Library", "Prompts"] },
    { title: "Exports", items: ["ChatGPT", "Claude", "Gemini", "OpenClaw", "Ollama"] },
  ];

  return (
    <aside className="sidebar">
      <div className="brand">
        <div className="brandMark">
          <Brain size={22} />
        </div>
        <div>
          <strong>AgentVM</strong>
          <span>Studio</span>
        </div>
      </div>
      {groups.map((group) => (
        <nav key={group.title}>
          <span>{group.title}</span>
          {group.items.map((item) => (
            <button
              key={item}
              className={item === activeTab ? "navItem active" : "navItem"}
              onClick={() => onChange(item)}
            >
              <ChevronRight size={15} />
              {item}
            </button>
          ))}
        </nav>
      ))}
      <div className="daemon">
        <span />
        AgentVM daemon ready
      </div>
    </aside>
  );
}

function Pipeline({ packageState }: { packageState: number }) {
  const steps = [
    ["Assemble", "Collect brain components"],
    ["Validate", "Check manifest schema"],
    ["Optimize", "Deduplicate memories"],
    ["Package", "Create Agent Image"],
    ["Export", "Generate platform bundle"],
  ];

  return (
    <section className="pipeline">
      <div className="panelHeader">
        <h2>Package Pipeline</h2>
        <Archive size={17} />
      </div>
      <div className="pipelineSteps">
        {steps.map(([title, caption], index) => {
          const state = index + 1;
          return (
            <div className={state <= packageState ? "pipeStep done" : "pipeStep"} key={title}>
              <span>{state <= packageState ? <CheckCircle2 size={16} /> : state}</span>
              <strong>{title}</strong>
              <small>{caption}</small>
            </div>
          );
        })}
      </div>
    </section>
  );
}

function Metric({ label, value }: { label: string; value: number }) {
  return (
    <div className="metric">
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}

function StatusLine({ ok, label }: { ok: boolean; label: string }) {
  return (
    <div className={ok ? "status ok" : "status bad"}>
      {ok ? <ShieldCheck size={20} /> : <CircleAlert size={20} />}
      <span>{label}</span>
    </div>
  );
}

function validateAgent(agent: AgentImage) {
  const errors: string[] = [];
  const warnings: string[] = [];
  if (agent.apiVersion !== "agentvm/v1") errors.push("apiVersion must be agentvm/v1");
  if (agent.kind !== "AgentImage") errors.push("kind must be AgentImage");
  if (!agent.metadata?.name) errors.push("metadata.name is required");
  if (!agent.metadata?.version) errors.push("metadata.version is required");
  if (!agent.identity?.persona) warnings.push("Add identity.persona for better platform exports.");
  if (!agent.runtime?.preferredModels?.length) warnings.push("Add runtime.preferredModels for runtime portability.");
  return { errors, warnings };
}

function repairImage(agent: AgentImage) {
  const next = cloneAgent(agent);
  next.apiVersion = "agentvm/v1";
  next.kind = "AgentImage";
  const metadata = ensureMetadata(next);
  metadata.name = metadata.name ? slugify(metadata.name) : slugify(metadata.displayName ?? next.identity?.name ?? "portable-agent");
  metadata.version = metadata.version || "1.0.0";
  const identity = ensureIdentity(next);
  identity.persona =
    identity.persona ||
    "A portable AgentVM assistant with user-owned memory, preferences, skills, and runtime context.";
  const runtime = ensureRuntime(next);
  if (!runtime.preferredModels?.length) {
    runtime.preferredModels = [{ provider: "local", model: "portable-default", priority: 1 }];
  }
  return next;
}

function countMemory(agent: AgentImage) {
  return ["episodic", "semantic", "procedural", "social"].filter((key) => Boolean(agent.memory?.[key])).length;
}

function countSkills(agent: AgentImage) {
  return (agent.skills?.builtin?.length ?? 0) + (agent.skills?.registry?.length ?? 0);
}

function platformReadiness(agent: AgentImage): Platform[] {
  const hasPersona = Boolean(agent.identity?.persona);
  const hasMemory = countMemory(agent) > 0;
  const hasSkills = countSkills(agent) > 0;

  return [
    {
      key: "chatgpt",
      label: "ChatGPT",
      output: "custom-instructions.md + knowledge-base.md + gpt-config.json",
      ready: hasPersona,
      note: "Needs persona",
    },
    {
      key: "claude",
      label: "Claude",
      output: "project-instructions.md + project-knowledge.md + skills/",
      ready: hasPersona && hasMemory,
      note: "Needs memory",
    },
    {
      key: "gemini",
      label: "Gemini",
      output: "gem-instructions.md + knowledge bundle",
      ready: hasPersona,
      note: "Needs persona",
    },
    {
      key: "openclaw",
      label: "OpenClaw",
      output: "SOUL.md + USER.md + AGENTS.md + MEMORY.md + skills/",
      ready: hasPersona && hasMemory && hasSkills,
      note: "Needs skills",
    },
    {
      key: "ollama",
      label: "Ollama",
      output: "Modelfile + system-prompt.md + context/",
      ready: hasPersona,
      note: "Needs persona",
    },
  ];
}

function scorePortability(agent: AgentImage, platforms: Platform[]) {
  let score = 35;
  if (agent.identity?.persona) score += 20;
  if (countMemory(agent) >= 4) score += 15;
  if (countSkills(agent)) score += 10;
  if (agent.tools?.security) score += 8;
  if (agent.runtime?.preferredModels?.length) score += 7;
  score += platforms.filter((platform) => platform.ready).length;
  return Math.min(score, 100);
}

function buildPlatformExport(
  agent: AgentImage,
  platform: string,
  packageFiles: PackageFiles = buildDefaultPackageFiles(),
): PlatformExport {
  const name = agent.metadata?.name ?? "agent";
  const persona = agent.identity?.persona ?? "A helpful AI assistant.";
  const skills = [
    ...(agent.skills?.builtin ?? []).map((skill) => `${skill.id}@${skill.version ?? "unspecified"}`),
    ...(agent.skills?.registry ?? []).map((skill) => `${skill.id}@${skill.version}`),
  ];
  const memoryMarkdown = buildMemoryMarkdown(packageFiles);
  const memorySources = Object.entries(agent.memory ?? {})
    .filter(([key]) => key !== "strategy")
    .map(([key, value]) => `${key}: ${JSON.stringify(value)}`);
  const knowledge = [
    `# ${agent.metadata?.displayName ?? name}`,
    "",
    agent.metadata?.description ?? "Portable AgentVM image.",
    "",
    "## Persona",
    persona,
    "",
    "## Memory Sources",
    memorySources.length ? memorySources.map((source) => `- ${source}`).join("\n") : "- none",
    "",
    "## Skills",
    skills.length ? skills.map((skill) => `- ${skill}`).join("\n") : "- none",
    "",
    "## Packaged Memory",
    memoryMarkdown,
  ].join("\n");

  if (platform === "ollama") {
    const files = {
      "Modelfile": `FROM llama3.2

SYSTEM """
${persona.replace(/"""/g, '\\"\\"\\"')}
"""

# AgentVM context
${JSON.stringify(agent, null, 2)}
`,
      "context/agentvm.json": JSON.stringify(agent, null, 2),
      ...prefixFiles(packageFiles, "context/"),
    };
    return {
      platform,
      filename: "Modelfile",
      mime: "text/plain",
      preview: "Modelfile + AgentVM context + packaged memory files",
      content: files["Modelfile"],
      files,
      warnings: platformWarnings(agent, platform),
    };
  }

  if (platform === "openclaw") {
    const files = {
      "SOUL.md": `# SOUL.md\n\n${persona}\n`,
      "AGENTS.md": [
        "# AGENTS.md",
        "",
        "Follow the AgentVM exported memory files and preserve user-owned context portability.",
        "Keep platform-specific edits synchronized back into the AgentVM image when possible.",
        "",
      ].join("\n"),
      "MEMORY.md": knowledge,
      ...packageFiles,
    };
    return {
      platform,
      filename: "SOUL.md",
      mime: "text/markdown",
      preview: "SOUL.md + AGENTS.md + MEMORY.md + memory/ files",
      content: files["SOUL.md"],
      files,
      warnings: platformWarnings(agent, platform),
    };
  }

  if (platform === "claude") {
    const files = {
      "project-instructions.md": `# Project Instructions\n\n${persona}\n`,
      "project-knowledge.md": knowledge,
      ...prefixFiles(packageFiles, "knowledge/"),
    };
    return {
      platform,
      filename: "project-instructions.md",
      mime: "text/markdown",
      preview: "Claude project instructions + project knowledge + memory knowledge files",
      content: files["project-instructions.md"],
      files,
      warnings: platformWarnings(agent, platform),
    };
  }

  if (platform === "gemini") {
    const files = {
      "gem-instructions.md": `# Gem Instructions\n\n${persona}\n`,
      "knowledge-bundle.md": knowledge,
      "gem-config.json": JSON.stringify(
        {
          name: agent.metadata?.displayName ?? name,
          description: agent.metadata?.description,
          instructionsFile: "gem-instructions.md",
          knowledgeFile: "knowledge-bundle.md",
        },
        null,
        2,
      ),
    };
    return {
      platform,
      filename: "gem-instructions.md",
      mime: "text/markdown",
      preview: "Gem instructions + knowledge bundle + config",
      content: files["gem-instructions.md"],
      files,
      warnings: platformWarnings(agent, platform),
    };
  }

  const config = {
    name: agent.metadata?.displayName ?? name,
    description: agent.metadata?.description,
    instructions: persona,
    knowledge,
    deniedTools: agent.tools?.denied ?? [],
    preferredModels: agent.runtime?.preferredModels ?? [],
  };
  const files = {
    "custom-instructions.md": `# Custom Instructions\n\n${persona}\n`,
    "knowledge-base.md": knowledge,
    "gpt-config.json": JSON.stringify(config, null, 2),
  };
  return {
    platform,
    filename: "gpt-config.json",
    mime: "application/json",
    preview: "Custom instructions + knowledge base + GPT config",
    content: files["gpt-config.json"],
    files,
    warnings: platformWarnings(agent, platform),
  };
}

function buildBrowserPackage(agent: AgentImage, manifest: string, packageFiles: PackageFiles = buildDefaultPackageFiles()) {
  const name = agent.metadata?.name ?? "agent";
  const packagedAt = new Date().toISOString();
  const files = {
    ...buildDefaultPackageFiles(),
    ...packageFiles,
    "agent.yaml": manifest,
    "meta/package.json": JSON.stringify(
      {
        format: "agentvm-browser-bundle",
        packagedAt,
        entrypoint: "agent.yaml",
        generatedBy: "AgentVM Studio",
      },
      null,
      2,
    ),
  };
  return {
    filename: `${name}.agentvm.json`,
    content: {
      package: {
        format: "agentvm-browser-bundle",
        version: "0.1.0",
        packagedAt,
        entrypoint: "agent.yaml",
      },
      manifest: agent,
      files,
    },
  };
}

function isBrowserPackage(value: unknown): value is {
  package?: { format?: string };
  manifest?: AgentImage;
  agent?: AgentImage;
  files?: PackageFiles;
} {
  if (!value || typeof value !== "object") return false;
  const candidate = value as { manifest?: unknown; agent?: unknown };
  return isAgentImage(candidate.manifest) || isAgentImage(candidate.agent);
}

function isAgentImage(value: unknown): value is AgentImage {
  return Boolean(value && typeof value === "object" && "metadata" in value);
}

function defaultStudioSnapshot(): StudioSnapshot {
  return buildStudioSnapshot({
    agent: cloneAgent(sampleAgent),
    manifest: dump(sampleAgent, { lineWidth: 88 }),
    packageFiles: buildDefaultPackageFiles(),
    comparisonManifest: dump(buildComparisonSeed(sampleAgent), { lineWidth: 88 }),
    activeTab: "Dashboard",
    lastImport: "sample/turkish-dev.yaml",
    selectedPlatform: "openclaw",
    packageState: 3,
    registryUrl: "http://127.0.0.1:8787",
    registryOwner: "local",
    registryQuery: "",
  });
}

function buildStudioSnapshot(
  input: Omit<StudioSnapshot, "version" | "savedAt">,
): StudioSnapshot {
  return {
    version: 1,
    ...input,
    savedAt: new Date().toISOString(),
  };
}

function loadStudioSnapshot(): StudioSnapshot | null {
  if (typeof window === "undefined") return null;
  try {
    const raw = window.localStorage.getItem(studioStorageKey);
    if (!raw) return null;
    const snapshot = JSON.parse(raw) as Partial<StudioSnapshot>;
    if (!isValidStudioSnapshot(snapshot)) return null;
    return snapshot;
  } catch {
    return null;
  }
}

function saveStudioSnapshot(snapshot: StudioSnapshot) {
  try {
    window.localStorage.setItem(studioStorageKey, JSON.stringify(snapshot));
  } catch {
    // Local autosave is best-effort; downloads and registry publishing still work without it.
  }
}

function clearStudioSnapshot() {
  try {
    window.localStorage.removeItem(studioStorageKey);
  } catch {
    // Ignore storage failures so reset remains usable in restricted browsers.
  }
}

function isValidStudioSnapshot(snapshot: Partial<StudioSnapshot>): snapshot is StudioSnapshot {
  return Boolean(
    snapshot.version === 1 &&
      isAgentImage(snapshot.agent) &&
      typeof snapshot.manifest === "string" &&
      isPackageFiles(snapshot.packageFiles) &&
      typeof snapshot.comparisonManifest === "string" &&
      typeof snapshot.activeTab === "string" &&
      typeof snapshot.lastImport === "string" &&
      typeof snapshot.selectedPlatform === "string" &&
      typeof snapshot.packageState === "number" &&
      typeof snapshot.registryUrl === "string" &&
      typeof snapshot.registryOwner === "string" &&
      typeof snapshot.registryQuery === "string" &&
      typeof snapshot.savedAt === "string",
  );
}

function isPackageFiles(value: unknown): value is PackageFiles {
  return Boolean(
    value &&
      typeof value === "object" &&
      Object.entries(value as Record<string, unknown>).every(
        ([path, content]) => typeof path === "string" && typeof content === "string",
      ),
  );
}

function formatSavedAt(value: string) {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return "now";
  return date.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit", second: "2-digit" });
}

function importPlatformText(
  platform: string,
  displayName: string,
  instructions: string,
  memoryNotes: string,
): PlatformImportResult {
  const label = platformLabel(platform);
  const trimmedInstructions = instructions.trim();
  const trimmedMemory = memoryNotes.trim();
  const name = slugify(displayName || `${label} Agent`) || "imported-agent";
  const model = importModelTarget(platform);
  const agent: AgentImage = {
    apiVersion: "agentvm/v1",
    kind: "AgentImage",
    metadata: {
      name,
      version: "1.0.0",
      displayName: displayName.trim() || `${label} Agent`,
      description: `AgentVM image imported from ${label} platform instructions.`,
      tags: ["imported", platform, "portable"],
    },
    identity: {
      name: displayName.trim() || `${label} Agent`,
      persona:
        firstParagraph(trimmedInstructions) ||
        `Portable assistant imported from ${label} platform instructions.`,
    },
    memory: cloneData(sampleAgent.memory),
    skills: {
      builtin: [
        { id: "migration-review", version: "1.0.0", enabled: true },
        { id: `${platform}-adapter`, version: "1.0.0", enabled: true },
      ],
    },
    tools: {
      denied: ["network:untrusted"],
      security: {
        importSource: platform,
        preserveUserOwnedMemory: true,
      },
    },
    runtime: {
      preferredModels: [model],
    },
    export: cloneData(sampleAgent.export),
  };
  const files = buildDefaultPackageFiles();
  files["memory/episodic.md"] = [
    "# Episodic Memory",
    "",
    `- Imported from ${label} instructions through AgentVM Studio.`,
    trimmedMemory ? `- Preserved owner note: ${trimmedMemory.replace(/\s+/g, " ")}` : "",
    "",
  ]
    .filter(Boolean)
    .join("\n");
  files["memory/semantic.json"] = JSON.stringify(
    {
      facts: [
        `Source platform: ${label}`,
        `Imported instructions length: ${trimmedInstructions.length} characters`,
      ],
      preferences: trimmedMemory ? [trimmedMemory] : ["Preserve user-owned agent context across platforms."],
    },
    null,
    2,
  );
  files["memory/procedural.yaml"] = [
    "skills:",
    "  - id: migration-review",
    "    when: checking whether platform-local behavior survived the import",
    `  - id: ${platform}-adapter`,
    `    when: exporting back to ${label}`,
    "",
  ].join("\n");
  files["prompts/imported-instructions.md"] = [`# Imported ${label} Instructions`, "", trimmedInstructions].join("\n");
  files["meta/import-source.json"] = JSON.stringify(
    {
      sourcePlatform: platform,
      importedAt: new Date().toISOString(),
      instructionsFile: "prompts/imported-instructions.md",
      memoryFile: "memory/episodic.md",
    },
    null,
    2,
  );
  return {
    agent,
    files,
    source: `platform-import/${platform}`,
  };
}

function platformImportExample(platform: string) {
  if (platform === "claude") {
    return "You are a project assistant who knows this repository, preserves concise engineering context, and cites exact files when making changes.";
  }
  if (platform === "gemini") {
    return "Gem instructions\n\nYou are a cross-platform research and coding assistant. Preserve owner preferences, knowledge files, and portable AgentVM memory when moving into or out of Gemini.";
  }
  if (platform === "openclaw") {
    return "SOUL.md\n\nYou are the owner's portable coding agent. Keep memory, skills, and repo-specific preferences synchronized back to AgentVM.";
  }
  if (platform === "ollama") {
    return 'SYSTEM """\nYou are a local-first assistant. Prefer offline execution, privacy, and portable context files.\n"""';
  }
  return "Custom Instructions\n\nBe direct, remember my preferences, preserve project context, and verify work with real commands before claiming success.";
}

function importModelTarget(platform: string) {
  if (platform === "claude") return { provider: "anthropic", model: "claude-sonnet-4", priority: 1 };
  if (platform === "gemini") return { provider: "gemini", model: "gemini-pro", priority: 1 };
  if (platform === "ollama") return { provider: "local", model: "llama3.2", priority: 1 };
  if (platform === "openclaw") return { provider: "openclaw", model: "portable-default", priority: 1 };
  return { provider: "openai", model: "gpt-4o", priority: 1 };
}

function firstParagraph(text: string) {
  return text
    .split(/\n\s*\n/)
    .map((part) => part.trim())
    .find((part) => part && !/^custom instructions$/i.test(part) && !/^soul\.md$/i.test(part))
    ?.replace(/^SYSTEM\s+"""\s*/i, "")
    .replace(/"""$/g, "")
    .trim();
}

function buildDefaultPackageFiles(): PackageFiles {
  return {
    "memory/episodic.md": [
      "# Episodic Memory",
      "",
      "- 2026-06-26: User wants a portable agent brain that can move between platforms without losing memory.",
      "- 2026-06-26: Prefer direct, evidence-backed engineering work over broad architecture talk.",
      "",
    ].join("\n"),
    "memory/semantic.json": JSON.stringify(
      {
        facts: [
          "AgentVM packages memory, personality, skills, tools, and runtime preferences into a portable image.",
        ],
        preferences: ["Keep user-owned agent context portable across platforms."],
      },
      null,
      2,
    ),
    "memory/procedural.yaml": "skills:\n  - id: code-review\n    when: validating implementation changes\n",
    "memory/social.yaml": "contacts:\n  - id: owner\n    relationship: primary-user\n",
  };
}

function normalizePackageFilePath(path: string) {
  const normalized = path.trim().replace(/\\/g, "/").replace(/^\.\/+/, "");
  if (!normalized) return "";
  if (normalized.startsWith("/") || normalized.includes("://")) return "";
  if (normalized.split("/").some((part) => part === "" || part === "." || part === "..")) return "";
  if (!/^[A-Za-z0-9._/-]+$/.test(normalized)) return "";
  return normalized;
}

function defaultPackageFileContent(path: string, kind: string) {
  if (kind === "json" || path.endsWith(".json")) {
    return JSON.stringify({ notes: [] }, null, 2);
  }
  if (kind === "skill" || path.startsWith("skills/")) {
    const parts = path.split("/").filter(Boolean);
    const id = parts.length > 1 ? parts[parts.length - 2] : parts[0]?.replace(/\.[^.]+$/, "") ?? "custom-skill";
    return [`# ${titleize(id)}`, "", "Use this skill when the owner needs this packaged behavior.", ""].join("\n");
  }
  if (kind === "prompt" || path.startsWith("prompts/")) {
    return [`# ${titleize(path.split("/").pop()?.replace(/\.[^.]+$/, "") ?? "Prompt")}`, "", "Add reusable prompt context here.", ""].join("\n");
  }
  return ["# Memory Note", "", "- Add portable context that should travel with this agent.", ""].join("\n");
}

function titleize(value: string) {
  return value
    .replace(/[-_]+/g, " ")
    .replace(/\b\w/g, (letter) => letter.toUpperCase());
}

function templateAgent(
  name: string,
  displayName: string,
  description: string,
  tags: string[],
  persona: string,
  skillIds: string[],
  preferredModels: NonNullable<AgentImage["runtime"]>["preferredModels"],
  deniedTools: string[] = [],
): AgentImage {
  return {
    apiVersion: "agentvm/v1",
    kind: "AgentImage",
    metadata: {
      name,
      version: "1.0.0",
      displayName,
      description,
      tags,
    },
    identity: {
      name: displayName,
      persona,
    },
    memory: cloneData(sampleAgent.memory),
    skills: {
      builtin: skillIds.map((id) => ({ id, version: "1.0.0", enabled: true })),
    },
    tools: deniedTools.length
      ? {
          denied: deniedTools,
        }
      : undefined,
    runtime: {
      preferredModels,
    },
    export: cloneData(sampleAgent.export),
  };
}

function createTemplateSeed(agent: AgentImage, category: string, summary: string): TemplateSeed {
  const name = agent.metadata?.name ?? "agent";
  const files = buildDefaultPackageFiles();
  files["memory/episodic.md"] = [
    "# Episodic Memory",
    "",
    `- Template selected: ${agent.metadata?.displayName ?? name}.`,
    `- Primary use case: ${summary}`,
    "",
  ].join("\n");
  files["memory/semantic.json"] = JSON.stringify(
    {
      facts: [agent.metadata?.description ?? "Portable AgentVM template."],
      preferences: [`Use ${agent.metadata?.displayName ?? name} defaults unless the owner changes them.`],
    },
    null,
    2,
  );
  files["memory/procedural.yaml"] = `skills:\n${(agent.skills?.builtin ?? [])
    .map((skill) => `  - id: ${skill.id}\n    when: ${skill.id.replace(/-/g, " ")}`)
    .join("\n") || "  []"}\n`;
  files["memory/social.yaml"] = "contacts:\n  - id: owner\n    relationship: primary-user\n";
  return {
    id: name,
    label: agent.metadata?.displayName ?? name,
    category,
    summary,
    agent,
    files,
  };
}

function extractEditablePackageFiles(files?: PackageFiles): PackageFiles {
  const next = buildDefaultPackageFiles();
  if (!files) return next;
  for (const [path, content] of Object.entries(files)) {
    if (path === "agent.yaml" || path === "meta/package.json") continue;
    if (typeof content === "string") next[path] = content;
  }
  return next;
}

function isMemoryTab(tab: string) {
  return tab === "Core Memory" || tab === "Episodic" || tab === "Knowledge";
}

function summarizeMemoryFile(content: string) {
  const trimmed = content.trim();
  if (!trimmed) return "empty";
  const firstLine = trimmed.split("\n").find((line) => line.trim()) ?? "";
  return firstLine.replace(/^#+\s*/, "").slice(0, 86);
}

function isExportTab(tab: string) {
  return ["ChatGPT", "Claude", "Gemini", "OpenClaw", "Ollama"].includes(tab);
}

function platformKeyFromTab(tab: string) {
  const map: Record<string, string> = {
    ChatGPT: "chatgpt",
    Claude: "claude",
    Gemini: "gemini",
    OpenClaw: "openclaw",
    Ollama: "ollama",
  };
  return map[tab];
}

function platformTabFromKey(platform: string) {
  const map: Record<string, string> = {
    chatgpt: "ChatGPT",
    claude: "Claude",
    gemini: "Gemini",
    openclaw: "OpenClaw",
    ollama: "Ollama",
  };
  return map[platform] ?? "ChatGPT";
}

function platformLabel(platform: string) {
  return platformTabFromKey(platform);
}

function prefixFiles(files: PackageFiles, prefix: string): PackageFiles {
  return Object.fromEntries(Object.entries(files).map(([path, content]) => [`${prefix}${path}`, content]));
}

function buildMemoryMarkdown(files: PackageFiles) {
  return Object.entries(files)
    .sort(([left], [right]) => left.localeCompare(right))
    .map(([path, content]) => [`### ${path}`, "", content.trim() || "(empty)"].join("\n"))
    .join("\n\n");
}

function platformWarnings(agent: AgentImage, platform: string) {
  const warnings: string[] = [];
  if (!agent.identity?.persona) warnings.push("Missing persona will make platform instructions weak.");
  if (countMemory(agent) === 0) warnings.push("No manifest memory sources are enabled.");
  if (platform === "chatgpt" && (agent.tools?.denied?.length ?? 0) > 0) {
    warnings.push("ChatGPT export records denied tools as guidance, not an enforced runtime policy.");
  }
  if (platform === "ollama" && !agent.runtime?.preferredModels?.some((model) => model.provider === "local")) {
    warnings.push("Ollama export uses a generic local base model unless a local model target is configured.");
  }
  return warnings;
}

function buildExportBundle(exportPackage: PlatformExport) {
  return {
    package: {
      format: "agentvm-platform-export",
      platform: exportPackage.platform,
      generatedAt: new Date().toISOString(),
      entrypoint: exportPackage.filename,
    },
    files: exportPackage.files,
    warnings: exportPackage.warnings,
  };
}

function scanPackageSecurity(manifest: string, files: PackageFiles): SecurityFinding[] {
  const scanTarget = {
    ...files,
    "agent.yaml": manifest,
  };
  const findings: SecurityFinding[] = [];
  for (const [path, content] of Object.entries(scanTarget).sort(([left], [right]) => left.localeCompare(right))) {
    if (!isTextPath(path)) continue;
    content.split(/\r?\n/).forEach((line, index) => {
      const match = secretLikeMatch(line);
      if (!match) return;
      findings.push({
        severity: "high",
        path,
        line: index + 1,
        rule: match.rule,
        message: match.message,
      });
    });
  }
  return findings;
}

function secretLikeMatch(line: string): { rule: string; message: string } | null {
  const lower = line.toLowerCase();
  if (lower.includes("example") || lower.includes("placeholder") || lower.includes("redacted")) {
    return null;
  }
  if (/-----BEGIN [A-Z ]*PRIVATE KEY-----/.test(line)) {
    return { rule: "private-key", message: "Private key material detected" };
  }
  if (/\bsk-[A-Za-z0-9_-]{16,}\b/.test(line)) {
    return { rule: "openai-key", message: "OpenAI-style API key detected" };
  }
  if (/\b(ghp_[A-Za-z0-9_]{20,}|github_pat_[A-Za-z0-9_]{20,})\b/.test(line)) {
    return { rule: "github-token", message: "GitHub token detected" };
  }
  if (/\bxox[abprs]-[A-Za-z0-9-]{16,}\b/.test(line)) {
    return { rule: "slack-token", message: "Slack token detected" };
  }
  if (/\bAKIA[A-Z0-9]{16}\b/.test(line)) {
    return { rule: "aws-access-key", message: "AWS access key detected" };
  }
  if (hasSecretAssignment(line)) {
    return { rule: "secret-assignment", message: "Secret-like assignment detected" };
  }
  return null;
}

function hasSecretAssignment(line: string) {
  const names = [
    "api_key",
    "apikey",
    "api-key",
    "access_token",
    "auth_token",
    "refresh_token",
    "secret_key",
    "client_secret",
    "password",
  ];
  const lower = line.toLowerCase();
  return names.some((name) => {
    const index = lower.indexOf(name);
    if (index === -1) return false;
    const after = line.slice(index + name.length).trimStart();
    if (!after.startsWith(":") && !after.startsWith("=")) return false;
    const value = after.slice(1).trim();
    return value.length >= 8 && !/^["']?\$?\{?[A-Z0-9_]+\}?["']?$/.test(value);
  });
}

function isTextPath(path: string) {
  return /\.(yaml|yml|json|md|txt|toml|env|ini|conf|cfg|modelfile)$/i.test(path) || path === "Modelfile";
}

function parseAgentManifest(text: string): { agent?: AgentImage; error?: string } {
  try {
    const loaded = load(text);
    if (isBrowserPackage(loaded)) {
      const agent = loaded.manifest ?? loaded.agent;
      return agent ? { agent } : { error: "Bundle does not contain an Agent Image manifest." };
    }
    if (isAgentImage(loaded)) return { agent: loaded };
    return { error: "Comparison input is not an Agent Image manifest." };
  } catch (error) {
    return { error: error instanceof Error ? error.message : "Comparison manifest could not be parsed." };
  }
}

function compareAgentImages(current: AgentImage, comparison?: AgentImage): DiffSection[] {
  const sections: Array<{ key: DiffSectionKey; label: string }> = [
    { key: "metadata", label: "Metadata" },
    { key: "identity", label: "Identity" },
    { key: "memory", label: "Memory" },
    { key: "skills", label: "Skills" },
    { key: "tools", label: "Tools" },
    { key: "runtime", label: "Runtime" },
    { key: "export", label: "Export Rules" },
  ];
  if (!comparison) {
    return sections.map((section) => ({
      ...section,
      status: "unchanged",
      summary: "Load a valid comparison image to calculate differences.",
    }));
  }
  return sections.map((section) => {
    const currentValue = (current as Record<DiffSectionKey, unknown>)[section.key];
    const incomingValue = (comparison as Record<DiffSectionKey, unknown>)[section.key];
    const status = diffStatus(currentValue, incomingValue);
    return {
      ...section,
      status,
      summary: summarizeDiff(section.key, currentValue, incomingValue, status),
    };
  });
}

function diffStatus(currentValue: unknown, incomingValue: unknown): DiffSection["status"] {
  if (currentValue === undefined && incomingValue !== undefined) return "added";
  if (currentValue !== undefined && incomingValue === undefined) return "removed";
  if (stableStringify(currentValue) !== stableStringify(incomingValue)) return "changed";
  return "unchanged";
}

function summarizeDiff(
  section: DiffSectionKey,
  currentValue: unknown,
  incomingValue: unknown,
  status: DiffSection["status"],
) {
  if (status === "unchanged") return "No difference in this section.";
  if (status === "added") return "Comparison image adds this section.";
  if (status === "removed") return "Comparison image does not include this section.";

  if (section === "metadata") {
    const currentMetadata = currentValue as AgentImage["metadata"];
    const incomingMetadata = incomingValue as AgentImage["metadata"];
    return `${currentMetadata?.name ?? "unnamed"} ${currentMetadata?.version ?? "unversioned"} -> ${incomingMetadata?.name ?? "unnamed"} ${incomingMetadata?.version ?? "unversioned"}`;
  }
  if (section === "identity") {
    const currentIdentity = currentValue as AgentImage["identity"];
    const incomingIdentity = incomingValue as AgentImage["identity"];
    return `${currentIdentity?.name ?? "unnamed"} persona differs from ${incomingIdentity?.name ?? "unnamed"}.`;
  }
  if (section === "memory") {
    return `${countObjectKeys(currentValue)} source(s) -> ${countObjectKeys(incomingValue)} source(s).`;
  }
  if (section === "skills") {
    return `${countSkillReferences(currentValue)} skill(s) -> ${countSkillReferences(incomingValue)} skill(s).`;
  }
  if (section === "tools") {
    return `${countDeniedTools(currentValue)} denied tool(s) -> ${countDeniedTools(incomingValue)} denied tool(s).`;
  }
  if (section === "runtime") {
    return `${countModelTargets(currentValue)} model target(s) -> ${countModelTargets(incomingValue)} model target(s).`;
  }
  return "Platform export overrides differ.";
}

function buildComparisonSeed(agent: AgentImage): AgentImage {
  const next = cloneAgent(agent);
  const metadata = ensureMetadata(next);
  metadata.name = `${metadata.name ?? "agent"}-fork`;
  metadata.version = bumpPatchVersion(metadata.version ?? "1.0.0");
  metadata.displayName = `${metadata.displayName ?? metadata.name ?? "Agent"} Fork`;
  metadata.tags = Array.from(new Set([...(metadata.tags ?? []), "fork"]));
  const identity = ensureIdentity(next);
  identity.persona = `${identity.persona ?? "Portable agent."}\n\nFork note: prefers explicit migration checks before platform changes.`;
  const skills = ensureSkills(next);
  skills.builtin = [
    ...(skills.builtin ?? []).filter((skill) => skill.id !== "migration-review"),
    { id: "migration-review", version: "1.0.0", enabled: true },
  ];
  return next;
}

function stableStringify(value: unknown): string {
  return JSON.stringify(sortJson(value));
}

function sortJson(value: unknown): unknown {
  if (Array.isArray(value)) return value.map(sortJson);
  if (!value || typeof value !== "object") return value;
  return Object.fromEntries(
    Object.entries(value as Record<string, unknown>)
      .sort(([left], [right]) => left.localeCompare(right))
      .map(([key, entry]) => [key, sortJson(entry)]),
  );
}

function cloneData<T>(value: T): T {
  return JSON.parse(JSON.stringify(value)) as T;
}

function countObjectKeys(value: unknown) {
  return value && typeof value === "object" ? Object.keys(value).length : 0;
}

function countSkillReferences(value: unknown) {
  const skills = value as AgentImage["skills"] | undefined;
  return (skills?.builtin?.length ?? 0) + (skills?.registry?.length ?? 0);
}

function countDeniedTools(value: unknown) {
  const tools = value as AgentImage["tools"] | undefined;
  return tools?.denied?.length ?? 0;
}

function countModelTargets(value: unknown) {
  const runtime = value as AgentImage["runtime"] | undefined;
  return runtime?.preferredModels?.length ?? 0;
}

function bumpPatchVersion(version: string) {
  const parts = version.split(".").map((part) => Number.parseInt(part, 10));
  if (parts.length !== 3 || parts.some((part) => Number.isNaN(part))) return "1.0.1";
  parts[2] += 1;
  return parts.join(".");
}

function downloadText(filename: string, content: string, mime: string) {
  const blob = new Blob([content], { type: mime });
  const url = URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = url;
  link.download = filename;
  link.click();
  URL.revokeObjectURL(url);
}

function cloneAgent(agent: AgentImage): AgentImage {
  return JSON.parse(JSON.stringify(agent)) as AgentImage;
}

function ensureMetadata(agent: AgentImage) {
  agent.metadata ??= {};
  agent.metadata.version ??= "1.0.0";
  return agent.metadata;
}

function ensureIdentity(agent: AgentImage) {
  agent.identity ??= {};
  return agent.identity;
}

function ensureMemory(agent: AgentImage) {
  agent.memory ??= {};
  return agent.memory;
}

function ensureSkills(agent: AgentImage) {
  agent.skills ??= {};
  agent.skills.builtin ??= [];
  agent.skills.registry ??= [];
  return agent.skills;
}

function ensureTools(agent: AgentImage) {
  agent.tools ??= {};
  agent.tools.denied ??= [];
  return agent.tools;
}

function ensureRuntime(agent: AgentImage) {
  agent.runtime ??= {};
  agent.runtime.preferredModels ??= [];
  return agent.runtime;
}

function parseSkillReference(reference: string) {
  const trimmed = reference.trim();
  if (!trimmed) return null;
  if (trimmed.startsWith("registry://skills/")) {
    const raw = trimmed.replace("registry://skills/", "");
    const [id, version = "1.0.0"] = raw.split("@");
    return { kind: "registry" as const, id: slugify(id), version, source: trimmed };
  }
  return { kind: "builtin" as const, id: slugify(trimmed), version: "1.0.0" };
}

function slugify(value: string) {
  return value
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9-]+/g, "-")
    .replace(/^-+|-+$/g, "");
}

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
