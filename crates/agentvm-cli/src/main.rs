use std::{
    collections::{BTreeMap, HashMap},
    fs,
    io::{Read, Write},
    net::TcpStream,
    path::{Path, PathBuf},
};

use agentvm_core::{
    checksum, diff, merge, AgentImage, ImageDiff, ImageValidator, RegistrySkill, SkillEntry,
    ValidationReport,
};
use agentvm_memory::{consolidate, export_markdown, search, MemoryStore};
use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use flate2::{read::GzDecoder, write::GzEncoder, Compression};

#[derive(Debug, Parser)]
#[command(name = "agentvm")]
#[command(about = "Validate, inspect, and compare AgentVM images")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Create a new AgentVM image directory from a built-in template.
    Init {
        /// Output directory for the image.
        path: PathBuf,

        /// Built-in template to use.
        #[arg(long, value_enum, default_value_t = TemplateKind::SeniorDev)]
        template: TemplateKind,

        /// Overwrite existing files in the output directory.
        #[arg(long)]
        force: bool,
    },

    /// Pack an AgentVM image directory into a .agentvm archive.
    Pack {
        /// Directory containing agent.yaml.
        directory: PathBuf,

        /// Output .agentvm archive path.
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Unpack a .agentvm archive into a directory.
    Unpack {
        /// Archive path.
        archive: PathBuf,

        /// Output directory.
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Validate an AgentVM manifest YAML file.
    Validate {
        /// Path to an agent.yaml-compatible manifest.
        path: PathBuf,

        /// Treat warnings as a non-zero result.
        #[arg(long)]
        strict: bool,
    },

    /// Print a compact summary of an AgentVM manifest.
    Inspect {
        /// Path to an agent.yaml-compatible manifest.
        path: PathBuf,

        /// Output format.
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
    },

    /// Compare two AgentVM manifest YAML files.
    Diff {
        /// Original manifest path.
        left: PathBuf,

        /// Changed manifest path.
        right: PathBuf,

        /// Emit machine-readable JSON.
        #[arg(long)]
        json: bool,
    },

    /// Merge two AgentVM manifests into a new manifest.
    Merge {
        /// Base manifest path.
        left: PathBuf,

        /// Overlay manifest path.
        right: PathBuf,

        /// Output merged manifest path.
        #[arg(short, long)]
        output: PathBuf,
    },

    /// Compute a stable SHA-256 checksum for an AgentVM manifest.
    Checksum {
        /// Path to an agent.yaml-compatible manifest or image directory.
        path: PathBuf,
    },

    /// Export an AgentVM manifest into a platform-specific portable bundle.
    Export {
        /// Path to an agent.yaml-compatible manifest or image directory.
        path: PathBuf,

        /// Target platform.
        #[arg(long, value_enum)]
        to: ExportTarget,

        /// Output directory for generated files.
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Import platform-local agent files into an AgentVM image directory.
    Import {
        #[command(subcommand)]
        command: ImportCommands,
    },

    /// Compose a runtime turn from an AgentVM image without calling a provider.
    Run {
        /// Path to an agent.yaml-compatible manifest or image directory.
        path: PathBuf,

        /// Target platform context to compose.
        #[arg(long, value_enum)]
        platform: Option<ExportTarget>,

        /// User message for this runtime turn.
        #[arg(long)]
        prompt: Option<String>,

        /// Emit the composed runtime request as JSON.
        #[arg(long)]
        json: bool,

        /// Compose context locally without calling a live LLM provider.
        #[arg(long)]
        dry_run: bool,
    },

    /// Inspect and query memory files inside an AgentVM image directory.
    Memory {
        #[command(subcommand)]
        command: MemoryCommands,
    },

    /// List, add, and remove skills in an AgentVM manifest.
    Skills {
        #[command(subcommand)]
        command: SkillsCommands,
    },

    /// Bump an AgentVM image version.
    Version {
        #[command(subcommand)]
        command: VersionCommands,
    },

    /// Print an AgentVM image changelog.
    Changelog {
        /// AgentVM image directory, or its agent.yaml path.
        path: PathBuf,
    },

    /// Scan an AgentVM image for secret-like content before publishing.
    Security {
        #[command(subcommand)]
        command: SecurityCommands,
    },

    /// Publish and query a local AgentVM registry.
    Registry {
        #[command(subcommand)]
        command: RegistryCommands,
    },
}

#[derive(Debug, Subcommand)]
enum ImportCommands {
    /// Import a ChatGPT export workspace containing custom-instructions.md and knowledge-base.md.
    Chatgpt {
        /// ChatGPT export workspace directory.
        #[arg(long)]
        workspace: PathBuf,

        /// Output AgentVM image directory.
        #[arg(short, long)]
        output: PathBuf,

        /// Overwrite an existing output directory.
        #[arg(long)]
        force: bool,
    },

    /// Import a Claude export workspace containing project-instructions.md and project-knowledge.md.
    Claude {
        /// Claude export workspace directory.
        #[arg(long)]
        workspace: PathBuf,

        /// Output AgentVM image directory.
        #[arg(short, long)]
        output: PathBuf,

        /// Overwrite an existing output directory.
        #[arg(long)]
        force: bool,
    },

    /// Import a Gemini export workspace containing gem-instructions.md and knowledge-bundle.md.
    Gemini {
        /// Gemini export workspace directory.
        #[arg(long)]
        workspace: PathBuf,

        /// Output AgentVM image directory.
        #[arg(short, long)]
        output: PathBuf,

        /// Overwrite an existing output directory.
        #[arg(long)]
        force: bool,
    },

    /// Import an OpenClaw workspace containing SOUL.md, AGENTS.md, MEMORY.md, and skills/.
    Openclaw {
        /// OpenClaw workspace directory.
        #[arg(long)]
        workspace: PathBuf,

        /// Output AgentVM image directory.
        #[arg(short, long)]
        output: PathBuf,

        /// Overwrite an existing output directory.
        #[arg(long)]
        force: bool,
    },

    /// Import an Ollama export workspace containing Modelfile, system-prompt.md, and context/.
    Ollama {
        /// Ollama export workspace directory.
        #[arg(long)]
        workspace: PathBuf,

        /// Output AgentVM image directory.
        #[arg(short, long)]
        output: PathBuf,

        /// Overwrite an existing output directory.
        #[arg(long)]
        force: bool,
    },
}

#[derive(Debug, Subcommand)]
enum MemoryCommands {
    /// List loaded memory documents by kind and source.
    List {
        /// AgentVM image directory, or its agent.yaml path.
        path: PathBuf,
    },

    /// Search memory documents using local BM25-style recall.
    Search {
        /// AgentVM image directory, or its agent.yaml path.
        path: PathBuf,

        /// Search query.
        query: String,

        /// Maximum result count.
        #[arg(long, default_value_t = 5)]
        limit: usize,
    },

    /// Produce a consolidation summary for local memory files.
    Consolidate {
        /// AgentVM image directory, or its agent.yaml path.
        path: PathBuf,
    },

    /// Export all loaded memory as Markdown.
    Export {
        /// AgentVM image directory, or its agent.yaml path.
        path: PathBuf,

        /// Output Markdown file.
        #[arg(short, long)]
        output: PathBuf,
    },
}

#[derive(Debug, Subcommand)]
enum SecurityCommands {
    /// Scan text files in an AgentVM image for secret-like content.
    Scan {
        /// AgentVM image directory, agent.yaml path, or browser bundle JSON.
        path: PathBuf,

        /// Return a non-zero exit code when findings are present.
        #[arg(long)]
        strict: bool,
    },
}

#[derive(Debug, Subcommand)]
enum SkillsCommands {
    /// List built-in and registry skills declared by an image.
    List {
        /// AgentVM image directory, or its agent.yaml path.
        path: PathBuf,
    },

    /// Add or update a skill reference in an image manifest.
    Add {
        /// AgentVM image directory, or its agent.yaml path.
        path: PathBuf,

        /// Skill reference, such as registry://skills/github-advanced or ./skills/code-review.
        reference: String,

        /// Skill version to record when the reference does not include one.
        #[arg(long, default_value = "1.0.0")]
        version: String,

        /// Add local built-in skills as disabled.
        #[arg(long)]
        disabled: bool,
    },

    /// Remove a skill from built-in and registry skill lists by id.
    Remove {
        /// AgentVM image directory, or its agent.yaml path.
        path: PathBuf,

        /// Skill id to remove.
        id: String,
    },
}

#[derive(Debug, Subcommand)]
enum VersionCommands {
    /// Bump metadata.version and prepend a CHANGELOG.md entry.
    Bump {
        /// AgentVM image directory, or its agent.yaml path.
        path: PathBuf,

        /// Version component to bump.
        #[arg(value_enum)]
        level: BumpLevel,

        /// Changelog message for the new version.
        #[arg(short, long)]
        message: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
enum RegistryCommands {
    /// List images in a registry.
    List {
        /// Registry base URL.
        #[arg(long, default_value = "http://127.0.0.1:8787")]
        url: String,
    },

    /// Search registry images.
    Search {
        /// Search query.
        query: String,

        /// Registry base URL.
        #[arg(long, default_value = "http://127.0.0.1:8787")]
        url: String,
    },

    /// Push Agent Image metadata to a registry.
    Push {
        /// AgentVM image directory, or its agent.yaml path.
        path: PathBuf,

        /// Registry owner namespace.
        #[arg(long, default_value = "local")]
        owner: String,

        /// Registry base URL.
        #[arg(long, default_value = "http://127.0.0.1:8787")]
        url: String,
    },

    /// Pull registry image metadata by owner/name or owner/name:version.
    Pull {
        /// Registry reference, such as local/my-agent or local/my-agent:1.0.0.
        reference: String,

        /// Output JSON file. Prints to stdout when omitted.
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Registry base URL.
        #[arg(long, default_value = "http://127.0.0.1:8787")]
        url: String,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum BumpLevel {
    Patch,
    Minor,
    Major,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum TemplateKind {
    SeniorDev,
    CreativeWriter,
    Researcher,
    CustomerSupport,
    DataAnalyst,
    TurkishDev,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum ExportTarget {
    Chatgpt,
    Claude,
    Gemini,
    Openclaw,
    Ollama,
}

fn main() {
    match run() {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            eprintln!("error: {error:#}");
            std::process::exit(2);
        }
    }
}

fn run() -> Result<i32> {
    match Cli::parse().command {
        Commands::Init {
            path,
            template,
            force,
        } => {
            init_image(&path, template, force)?;
            println!("created {}", path.display());
            Ok(0)
        }
        Commands::Pack { directory, output } => {
            let archive = output.unwrap_or_else(|| default_archive_path(&directory));
            pack_image(&directory, &archive)?;
            println!("packed {} -> {}", directory.display(), archive.display());
            Ok(0)
        }
        Commands::Unpack { archive, output } => {
            let directory = output.unwrap_or_else(|| default_unpack_path(&archive));
            unpack_image(&archive, &directory)?;
            println!("unpacked {} -> {}", archive.display(), directory.display());
            Ok(0)
        }
        Commands::Validate { path, strict } => {
            let image = read_image(&path)?;
            let validator = if strict {
                ImageValidator::strict()
            } else {
                ImageValidator::new()
            };
            let report = validator.validate(&image)?;
            print!("{}", render_validation_report(&path, &report));
            Ok(validation_exit_code(&report, strict))
        }
        Commands::Inspect { path, format } => {
            let image = read_image(&path)?;
            match format {
                OutputFormat::Text => print!("{}", render_image_summary(&image)),
                OutputFormat::Json => println!("{}", image.to_json()?),
            }
            Ok(0)
        }
        Commands::Diff { left, right, json } => {
            let left_image = read_image(&left)?;
            let right_image = read_image(&right)?;
            let image_diff = diff(&left_image, &right_image);
            if json {
                println!("{}", serde_json::to_string_pretty(&image_diff)?);
            } else {
                print!("{}", render_diff(&image_diff));
            }
            Ok(if image_diff.identical { 0 } else { 1 })
        }
        Commands::Merge {
            left,
            right,
            output,
        } => {
            let left_image = read_image(&left)?;
            let right_image = read_image(&right)?;
            let merged = merge(&left_image, &right_image);
            let report = ImageValidator::new().validate(&merged)?;
            if !report.valid {
                anyhow::bail!("{}", render_validation_report(&output, &report));
            }
            std::fs::write(&output, merged.to_yaml()?)
                .with_context(|| format!("failed to write {}", output.display()))?;
            println!(
                "merged {} + {} -> {}",
                left.display(),
                right.display(),
                output.display()
            );
            Ok(0)
        }
        Commands::Checksum { path } => {
            let image = read_image(&path)?;
            println!("{}", checksum(&image)?);
            Ok(0)
        }
        Commands::Export { path, to, output } => {
            let image = read_image(&path)?;
            let directory = output.unwrap_or_else(|| {
                PathBuf::from(format!(
                    "{}-{}",
                    image.metadata.name,
                    export_target_slug(to)
                ))
            });
            export_image(&image, to, &directory)?;
            println!(
                "exported {} -> {}",
                image.metadata.name,
                directory.display()
            );
            Ok(0)
        }
        Commands::Import { command } => run_import_command(command),
        Commands::Run {
            path,
            platform,
            prompt,
            json,
            dry_run,
        } => {
            if !dry_run {
                anyhow::bail!(
                    "live provider execution is not implemented yet; pass --dry-run to compose a portable runtime request"
                );
            }
            let image = read_image(&path)?;
            let prompt = prompt.unwrap_or_else(|| "Hello from AgentVM.".to_string());
            let platform = platform.unwrap_or_else(|| infer_runtime_platform(&image));
            if json {
                println!("{}", render_runtime_json(&path, &image, platform, &prompt)?);
            } else {
                print!("{}", render_runtime_text(&path, &image, platform, &prompt));
            }
            Ok(0)
        }
        Commands::Memory { command } => run_memory_command(command),
        Commands::Skills { command } => run_skills_command(command),
        Commands::Version { command } => run_version_command(command),
        Commands::Changelog { path } => {
            print!("{}", read_changelog(&path)?);
            Ok(0)
        }
        Commands::Security { command } => run_security_command(command),
        Commands::Registry { command } => run_registry_command(command),
    }
}

fn run_import_command(command: ImportCommands) -> Result<i32> {
    match command {
        ImportCommands::Chatgpt {
            workspace,
            output,
            force,
        } => {
            import_platform_workspace(&workspace, &output, force, PlatformImportKind::Chatgpt)?;
            println!(
                "imported ChatGPT workspace {} -> {}",
                workspace.display(),
                output.display()
            );
            Ok(0)
        }
        ImportCommands::Claude {
            workspace,
            output,
            force,
        } => {
            import_platform_workspace(&workspace, &output, force, PlatformImportKind::Claude)?;
            println!(
                "imported Claude workspace {} -> {}",
                workspace.display(),
                output.display()
            );
            Ok(0)
        }
        ImportCommands::Gemini {
            workspace,
            output,
            force,
        } => {
            import_platform_workspace(&workspace, &output, force, PlatformImportKind::Gemini)?;
            println!(
                "imported Gemini workspace {} -> {}",
                workspace.display(),
                output.display()
            );
            Ok(0)
        }
        ImportCommands::Openclaw {
            workspace,
            output,
            force,
        } => {
            import_openclaw_workspace(&workspace, &output, force)?;
            println!(
                "imported OpenClaw workspace {} -> {}",
                workspace.display(),
                output.display()
            );
            Ok(0)
        }
        ImportCommands::Ollama {
            workspace,
            output,
            force,
        } => {
            import_platform_workspace(&workspace, &output, force, PlatformImportKind::Ollama)?;
            println!(
                "imported Ollama workspace {} -> {}",
                workspace.display(),
                output.display()
            );
            Ok(0)
        }
    }
}

fn run_security_command(command: SecurityCommands) -> Result<i32> {
    match command {
        SecurityCommands::Scan { path, strict } => {
            let findings = scan_image_security(&path)?;
            print!("{}", render_security_scan(&path, &findings));
            Ok(security_scan_exit_code(&findings, strict))
        }
    }
}

fn read_image(path: &Path) -> Result<AgentImage> {
    let manifest_path = if path.is_dir() {
        path.join("agent.yaml")
    } else {
        path.to_path_buf()
    };
    let manifest = std::fs::read_to_string(&manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;
    read_image_text(&manifest, path)
}

fn read_image_text(manifest: &str, path: &Path) -> Result<AgentImage> {
    if path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
    {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(manifest) {
            if let Some(image) = value.get("manifest").or_else(|| value.get("agent")) {
                return serde_json::from_value(image.clone()).with_context(|| {
                    format!("failed to parse bundled manifest in {}", path.display())
                });
            }
        }
    }

    AgentImage::from_yaml(manifest).with_context(|| format!("failed to parse {}", path.display()))
}

fn resolve_image_dir(path: &Path) -> Result<PathBuf> {
    if path.is_dir() {
        return Ok(path.to_path_buf());
    }

    if path.file_name().and_then(|name| name.to_str()) == Some("agent.yaml") {
        return path
            .parent()
            .map(Path::to_path_buf)
            .with_context(|| format!("{} has no parent directory", path.display()));
    }

    anyhow::bail!(
        "{} is not an image directory or agent.yaml path; unpack .agentvm archives first",
        path.display()
    );
}

fn resolve_manifest_path(path: &Path) -> Result<PathBuf> {
    if path.is_dir() {
        return Ok(path.join("agent.yaml"));
    }

    if path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| matches!(extension, "yaml" | "yml"))
    {
        return Ok(path.to_path_buf());
    }

    anyhow::bail!(
        "{} is not an image directory or YAML manifest path; unpack .agentvm archives first",
        path.display()
    );
}

fn write_valid_image(path: &Path, image: &AgentImage) -> Result<()> {
    let manifest_path = resolve_manifest_path(path)?;
    let report = ImageValidator::new().validate(image)?;
    if !report.valid {
        anyhow::bail!("{}", render_validation_report(&manifest_path, &report));
    }
    std::fs::write(&manifest_path, image.to_yaml()?)
        .with_context(|| format!("failed to write {}", manifest_path.display()))
}

fn registry_image_payload(path: &Path, owner: &str) -> Result<serde_json::Value> {
    let image = read_image(path)?;
    let manifest = serde_json::from_str::<serde_json::Value>(&image.to_json()?)?;
    let files = image_files_for_registry(path, &image)?;
    Ok(serde_json::json!({
        "owner": owner,
        "name": image.metadata.name,
        "version": image.metadata.version,
        "description": image.metadata.description,
        "tags": image.metadata.tags,
        "private": false,
        "manifest": manifest,
        "files": files
    }))
}

fn image_files_for_registry(path: &Path, image: &AgentImage) -> Result<BTreeMap<String, String>> {
    if is_json_bundle_path(path) {
        let text = fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let value: serde_json::Value = serde_json::from_str(&text)
            .with_context(|| format!("failed to parse {}", path.display()))?;
        if let Some(files) = value.get("files").and_then(|files| files.as_object()) {
            return json_files_to_map(files);
        }
    }

    if path.is_dir() {
        let mut files = BTreeMap::new();
        collect_text_files(path, path, &mut files)?;
        return Ok(files);
    }

    if path.file_name().and_then(|name| name.to_str()) == Some("agent.yaml") {
        let directory = path
            .parent()
            .with_context(|| format!("{} has no parent directory", path.display()))?;
        let mut files = BTreeMap::new();
        collect_text_files(directory, directory, &mut files)?;
        return Ok(files);
    }

    let mut files = BTreeMap::new();
    files.insert("agent.yaml".to_string(), image.to_yaml()?);
    Ok(files)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SecurityFinding {
    severity: &'static str,
    rule: &'static str,
    file: String,
    line: usize,
    message: &'static str,
}

fn scan_image_security(path: &Path) -> Result<Vec<SecurityFinding>> {
    let image = read_image(path)?;
    let files = image_files_for_registry(path, &image)?;
    let mut findings = Vec::new();

    for (file, content) in files {
        for (line_index, line) in content.lines().enumerate() {
            if let Some((rule, message)) = secret_like_match(line) {
                findings.push(SecurityFinding {
                    severity: "high",
                    rule,
                    file: file.clone(),
                    line: line_index + 1,
                    message,
                });
            }
        }
    }

    findings.sort_by(|left, right| {
        left.file
            .cmp(&right.file)
            .then(left.line.cmp(&right.line))
            .then(left.rule.cmp(right.rule))
    });
    Ok(findings)
}

fn secret_like_match(line: &str) -> Option<(&'static str, &'static str)> {
    let trimmed = line.trim();
    let lower = trimmed.to_ascii_lowercase();
    if trimmed.contains("-----BEGIN ") && trimmed.contains(" PRIVATE KEY-----") {
        return Some(("private-key", "private key material should not be packaged"));
    }
    if trimmed.contains("sk-") && trimmed.len() >= 24 {
        return Some(("openai-key", "OpenAI-style API key detected"));
    }
    if trimmed.contains("ghp_") || trimmed.contains("github_pat_") {
        return Some(("github-token", "GitHub token detected"));
    }
    if trimmed.contains("xoxb-") || trimmed.contains("xoxp-") {
        return Some(("slack-token", "Slack token detected"));
    }
    if trimmed.contains("AKIA") && trimmed.len() >= 20 {
        return Some(("aws-access-key", "AWS access key id detected"));
    }
    if has_secret_assignment(&lower) {
        return Some((
            "secret-assignment",
            "secret-like key/value assignment detected",
        ));
    }
    None
}

fn has_secret_assignment(lower: &str) -> bool {
    const KEYS: &[&str] = &[
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
    KEYS.iter().any(|key| {
        lower.contains(key)
            && (lower.contains(':') || lower.contains('='))
            && !lower.contains("example")
            && !lower.contains("placeholder")
            && !lower.contains("redacted")
    })
}

fn render_security_scan(path: &Path, findings: &[SecurityFinding]) -> String {
    if findings.is_empty() {
        return format!(
            "{}: security scan passed; no secret-like content found\n",
            path.display()
        );
    }

    let mut output = format!(
        "{}: {} security finding(s)\n",
        path.display(),
        findings.len()
    );
    for finding in findings {
        output.push_str(&format!(
            "{} {}:{} [{}] {}\n",
            finding.severity, finding.file, finding.line, finding.rule, finding.message
        ));
    }
    output
}

fn security_scan_exit_code(findings: &[SecurityFinding], strict: bool) -> i32 {
    if strict && !findings.is_empty() {
        1
    } else {
        0
    }
}

fn json_files_to_map(
    files: &serde_json::Map<String, serde_json::Value>,
) -> Result<BTreeMap<String, String>> {
    let mut output = BTreeMap::new();
    for (relative, content) in files {
        let content = content
            .as_str()
            .with_context(|| format!("bundle file {relative} is not a string"))?;
        safe_bundle_destination(Path::new("."), relative)?;
        output.insert(relative.clone(), content.to_string());
    }
    Ok(output)
}

fn collect_text_files(
    root: &Path,
    current: &Path,
    files: &mut BTreeMap<String, String>,
) -> Result<()> {
    for entry in
        fs::read_dir(current).with_context(|| format!("failed to read {}", current.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            collect_text_files(root, &path, files)?;
            continue;
        }
        if !file_type.is_file() {
            continue;
        }
        let relative = path
            .strip_prefix(root)
            .with_context(|| format!("failed to relativize {}", path.display()))?
            .to_string_lossy()
            .replace('\\', "/");
        safe_bundle_destination(Path::new("."), &relative)?;
        let content = fs::read_to_string(&path).with_context(|| {
            format!(
                "registry push only supports text image files; failed to read {}",
                path.display()
            )
        })?;
        files.insert(relative, content);
    }
    Ok(())
}

fn run_memory_command(command: MemoryCommands) -> Result<i32> {
    match command {
        MemoryCommands::List { path } => {
            let image_dir = resolve_image_dir(&path)?;
            let store = MemoryStore::load(&image_dir)?;
            print!("{}", render_memory_list(&store));
            Ok(0)
        }
        MemoryCommands::Search { path, query, limit } => {
            let image_dir = resolve_image_dir(&path)?;
            let store = MemoryStore::load(&image_dir)?;
            let hits = search(&store, &query, limit);
            print!("{}", render_search_hits(&hits));
            Ok(if hits.is_empty() { 1 } else { 0 })
        }
        MemoryCommands::Consolidate { path } => {
            let image_dir = resolve_image_dir(&path)?;
            let store = MemoryStore::load(&image_dir)?;
            print!("{}", render_consolidation_report(&store));
            Ok(0)
        }
        MemoryCommands::Export { path, output } => {
            let image_dir = resolve_image_dir(&path)?;
            let store = MemoryStore::load(&image_dir)?;
            std::fs::write(&output, export_markdown(&store))
                .with_context(|| format!("failed to write {}", output.display()))?;
            println!("exported memory -> {}", output.display());
            Ok(0)
        }
    }
}

fn run_skills_command(command: SkillsCommands) -> Result<i32> {
    match command {
        SkillsCommands::List { path } => {
            let image = read_image(&path)?;
            print!("{}", render_skills(&image));
            Ok(0)
        }
        SkillsCommands::Add {
            path,
            reference,
            version,
            disabled,
        } => {
            let mut image = read_image(&path)?;
            let added = add_skill_reference(&mut image, &reference, &version, disabled)?;
            write_valid_image(&path, &image)?;
            println!("added {added}");
            Ok(0)
        }
        SkillsCommands::Remove { path, id } => {
            let mut image = read_image(&path)?;
            let removed = remove_skill(&mut image, &id);
            if removed == 0 {
                anyhow::bail!("skill '{id}' was not found");
            }
            write_valid_image(&path, &image)?;
            println!("removed {removed} skill reference(s) for {id}");
            Ok(0)
        }
    }
}

fn run_version_command(command: VersionCommands) -> Result<i32> {
    match command {
        VersionCommands::Bump {
            path,
            level,
            message,
        } => {
            let mut image = read_image(&path)?;
            let previous = image.metadata.version.clone();
            let next = bump_version(&previous, level)?;
            image.metadata.version = next.clone();
            write_valid_image(&path, &image)?;
            prepend_changelog_entry(
                &path,
                &next,
                message
                    .as_deref()
                    .unwrap_or("Version bumped with AgentVM CLI."),
            )?;
            println!("bumped {} -> {}", previous, next);
            Ok(0)
        }
    }
}

fn run_registry_command(command: RegistryCommands) -> Result<i32> {
    match command {
        RegistryCommands::List { url } => {
            let body = registry_get(&url, "/v1/images")?;
            print!("{}", render_registry_images(&body)?);
            Ok(0)
        }
        RegistryCommands::Search { query, url } => {
            let body = registry_get(&url, &format!("/v1/images?q={}", url_encode(&query)))?;
            print!("{}", render_registry_images(&body)?);
            Ok(0)
        }
        RegistryCommands::Push { path, owner, url } => {
            let payload = registry_image_payload(&path, &owner)?;
            let body = registry_post(&url, "/v1/images", &serde_json::to_string(&payload)?)?;
            let published: serde_json::Value = serde_json::from_str(&body)?;
            println!(
                "pushed {}/{}:{}",
                published["owner"].as_str().unwrap_or("unknown"),
                published["name"].as_str().unwrap_or("unknown"),
                published["version"].as_str().unwrap_or("unknown")
            );
            Ok(0)
        }
        RegistryCommands::Pull {
            reference,
            output,
            url,
        } => {
            let reference = parse_registry_reference(&reference)?;
            let path = if let Some(version) = &reference.version {
                format!(
                    "/v1/images/{}/{}/{}",
                    reference.owner, reference.name, version
                )
            } else {
                format!("/v1/images/{}/{}", reference.owner, reference.name)
            };
            let body = registry_get(&url, &path)?;
            let pretty =
                serde_json::to_string_pretty(&serde_json::from_str::<serde_json::Value>(&body)?)?;
            if let Some(output) = output {
                fs::write(&output, format!("{pretty}\n"))
                    .with_context(|| format!("failed to write {}", output.display()))?;
                println!("pulled {} -> {}", reference.raw, output.display());
            } else {
                println!("{pretty}");
            }
            Ok(0)
        }
    }
}

fn init_image(path: &Path, template: TemplateKind, force: bool) -> Result<()> {
    let manifest_path = path.join("agent.yaml");
    if manifest_path.exists() && !force {
        anyhow::bail!(
            "{} already exists; pass --force to overwrite it",
            manifest_path.display()
        );
    }

    std::fs::create_dir_all(path.join("memory"))?;
    std::fs::create_dir_all(path.join("skills"))?;
    std::fs::create_dir_all(path.join("prompts"))?;
    std::fs::create_dir_all(path.join("tools"))?;
    std::fs::create_dir_all(path.join("meta"))?;

    let template = template_manifest(template);
    std::fs::write(&manifest_path, template.agent_yaml)?;
    std::fs::write(path.join("README.md"), template.readme)?;
    std::fs::write(
        path.join("CHANGELOG.md"),
        "# Changelog\n\n## 1.0.0\n- Initial AgentVM image.\n",
    )?;
    std::fs::write(path.join("memory/episodic.md"), "# Episodic Memory\n\n")?;
    let semantic_seed = serde_json::json!({
        "version": 1,
        "collections": {
            "agentProfile": {
                "entries": [
                    {
                        "key": "agent-purpose",
                        "value": template.system_prompt,
                        "confidence": 1.0,
                        "source": "template"
                    }
                ]
            }
        }
    });
    std::fs::write(
        path.join("memory/semantic.json"),
        serde_json::to_string_pretty(&semantic_seed)?,
    )?;
    std::fs::write(
        path.join("memory/procedural.yaml"),
        "version: 1\nskills: []\n",
    )?;
    std::fs::write(
        path.join("memory/social.yaml"),
        "version: 1\ncontacts: []\n",
    )?;
    std::fs::write(path.join("prompts/system.md"), template.system_prompt)?;
    std::fs::write(
        path.join("prompts/constraints.md"),
        "Never expose secrets. Keep user data portable and private.\n",
    )?;
    std::fs::write(path.join("tools/denied.yaml"), "denied:\n  - tts\n")?;

    let image = read_image(path)?;
    let report = ImageValidator::new().validate(&image)?;
    if !report.valid {
        anyhow::bail!("{}", render_validation_report(&manifest_path, &report));
    }

    Ok(())
}

fn import_openclaw_workspace(workspace: &Path, output: &Path, force: bool) -> Result<()> {
    if !workspace.is_dir() {
        anyhow::bail!("{} is not a workspace directory", workspace.display());
    }
    if output.exists() {
        if !force {
            anyhow::bail!(
                "{} already exists; pass --force to overwrite it",
                output.display()
            );
        }
        fs::remove_dir_all(output)
            .with_context(|| format!("failed to remove {}", output.display()))?;
    }

    fs::create_dir_all(output.join("memory"))?;
    fs::create_dir_all(output.join("skills"))?;
    fs::create_dir_all(output.join("prompts"))?;
    fs::create_dir_all(output.join("tools"))?;
    fs::create_dir_all(output.join("meta"))?;

    let name = slug_from_path(workspace);
    let soul = read_optional(workspace.join("SOUL.md"))?;
    let agents = read_optional(workspace.join("AGENTS.md"))?;
    let memory = read_optional(workspace.join("MEMORY.md"))?;
    let user = read_optional(workspace.join("USER.md"))?;
    let persona = first_non_empty(&[soul.as_deref(), agents.as_deref()])
        .unwrap_or("Imported OpenClaw agent")
        .to_string();
    let system_prompt = [soul.as_deref(), agents.as_deref()]
        .into_iter()
        .flatten()
        .filter(|value| !value.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n\n---\n\n");
    let skill_entries = import_openclaw_skills(workspace, output)?;

    fs::write(
        output.join("README.md"),
        render_import_readme(&name, workspace),
    )?;
    fs::write(
        output.join("CHANGELOG.md"),
        "# Changelog\n\n## 1.0.0\n- Imported from OpenClaw workspace.\n",
    )?;
    fs::write(output.join("prompts/system.md"), &system_prompt)?;
    fs::write(
        output.join("prompts/constraints.md"),
        "Imported from OpenClaw. Review memory before sharing this AgentVM image.\n",
    )?;
    fs::write(
        output.join("memory/episodic.md"),
        format!(
            "# Episodic Memory\n\n{}\n",
            user.unwrap_or_else(|| "No USER.md imported.".to_string())
        ),
    )?;
    fs::write(
        output.join("memory/semantic.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "version": 1,
            "collections": {
                "openclawImport": {
                    "entries": [
                        {
                            "key": "memory-md",
                            "value": memory.unwrap_or_else(|| "No MEMORY.md imported.".to_string()),
                            "confidence": 1.0,
                            "source": "OpenClaw MEMORY.md"
                        }
                    ]
                }
            }
        }))?,
    )?;
    fs::write(
        output.join("memory/procedural.yaml"),
        render_imported_procedural(&skill_entries),
    )?;
    fs::write(
        output.join("memory/social.yaml"),
        "version: 1\ncontacts: []\n",
    )?;
    fs::write(output.join("tools/denied.yaml"), "denied: []\n")?;
    fs::write(
        output.join("meta/provenance.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "importedFrom": workspace.display().to_string(),
            "platform": "openclaw"
        }))?,
    )?;

    let manifest = render_openclaw_import_manifest(&name, &persona, &skill_entries);
    fs::write(output.join("agent.yaml"), manifest)?;
    let image = read_image(output)?;
    let report = ImageValidator::new().validate(&image)?;
    if !report.valid {
        anyhow::bail!(
            "{}",
            render_validation_report(&output.join("agent.yaml"), &report)
        );
    }
    Ok(())
}

#[derive(Clone, Copy)]
enum PlatformImportKind {
    Chatgpt,
    Claude,
    Gemini,
    Ollama,
}

impl PlatformImportKind {
    fn label(self) -> &'static str {
        match self {
            PlatformImportKind::Chatgpt => "ChatGPT",
            PlatformImportKind::Claude => "Claude",
            PlatformImportKind::Gemini => "Gemini",
            PlatformImportKind::Ollama => "Ollama",
        }
    }

    fn slug(self) -> &'static str {
        match self {
            PlatformImportKind::Chatgpt => "chatgpt",
            PlatformImportKind::Claude => "claude",
            PlatformImportKind::Gemini => "gemini",
            PlatformImportKind::Ollama => "ollama",
        }
    }

    fn instruction_files(self) -> &'static [&'static str] {
        match self {
            PlatformImportKind::Chatgpt => &["custom-instructions.md"],
            PlatformImportKind::Claude => &["project-instructions.md"],
            PlatformImportKind::Gemini => &["gem-instructions.md"],
            PlatformImportKind::Ollama => &["system-prompt.md", "Modelfile"],
        }
    }

    fn knowledge_files(self) -> &'static [&'static str] {
        match self {
            PlatformImportKind::Chatgpt => &["knowledge-base.md", "gpt-config.json"],
            PlatformImportKind::Claude => &["project-knowledge.md"],
            PlatformImportKind::Gemini => &["knowledge-bundle.md", "gem-config.json"],
            PlatformImportKind::Ollama => &["context/agent.json", "Modelfile"],
        }
    }

    fn preferred_model(self) -> (&'static str, &'static str) {
        match self {
            PlatformImportKind::Chatgpt => ("openai", "gpt-4o"),
            PlatformImportKind::Claude => ("anthropic", "claude-sonnet-4"),
            PlatformImportKind::Gemini => ("gemini", "gemini-pro"),
            PlatformImportKind::Ollama => ("local", "llama3.2"),
        }
    }
}

fn import_platform_workspace(
    workspace: &Path,
    output: &Path,
    force: bool,
    platform: PlatformImportKind,
) -> Result<()> {
    if !workspace.is_dir() {
        anyhow::bail!("{} is not a workspace directory", workspace.display());
    }
    prepare_import_output(output, force)?;

    let name = slug_from_path(workspace);
    let instructions =
        read_first_existing(workspace, platform.instruction_files())?.with_context(|| {
            format!(
                "{} workspace is missing platform instructions",
                platform.label()
            )
        })?;
    let knowledge = read_named_files(workspace, platform.knowledge_files())?;
    let persona = import_persona(platform, &instructions);

    fs::write(
        output.join("README.md"),
        render_platform_import_readme(&name, workspace, platform),
    )?;
    fs::write(
        output.join("CHANGELOG.md"),
        format!(
            "# Changelog\n\n## 1.0.0\n- Imported from {} workspace.\n",
            platform.label()
        ),
    )?;
    fs::write(output.join("prompts/system.md"), &instructions)?;
    fs::write(
        output.join("prompts/constraints.md"),
        format!(
            "Imported from {}. Review memory before sharing this AgentVM image.\n",
            platform.label()
        ),
    )?;
    fs::write(
        output.join("memory/episodic.md"),
        format!(
            "# Episodic Memory\n\n- Imported from {} workspace `{}`.\n",
            platform.label(),
            workspace.display()
        ),
    )?;
    fs::write(
        output.join("memory/semantic.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "version": 1,
            "collections": {
                "platformImport": {
                    "entries": [
                        {
                            "key": "source-platform",
                            "value": platform.label(),
                            "confidence": 1.0,
                            "source": "AgentVM import"
                        },
                        {
                            "key": "knowledge",
                            "value": knowledge,
                            "confidence": 0.9,
                            "source": "platform export files"
                        }
                    ]
                }
            }
        }))?,
    )?;
    fs::write(
        output.join("memory/procedural.yaml"),
        format!(
            "version: 1\nskills:\n  - name: {}-migration-review\n    learned: imported\n    confidence: 0.8\n    trigger: platform migration\n",
            platform.slug()
        ),
    )?;
    fs::write(
        output.join("memory/social.yaml"),
        "version: 1\ncontacts: []\n",
    )?;
    fs::write(output.join("tools/denied.yaml"), "denied: []\n")?;
    fs::write(
        output.join("meta/provenance.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "importedFrom": workspace.display().to_string(),
            "platform": platform.slug()
        }))?,
    )?;

    let manifest = render_platform_import_manifest(&name, &persona, platform);
    fs::write(output.join("agent.yaml"), manifest)?;
    let image = read_image(output)?;
    let report = ImageValidator::new().validate(&image)?;
    if !report.valid {
        anyhow::bail!(
            "{}",
            render_validation_report(&output.join("agent.yaml"), &report)
        );
    }
    Ok(())
}

fn prepare_import_output(output: &Path, force: bool) -> Result<()> {
    if output.exists() {
        if !force {
            anyhow::bail!(
                "{} already exists; pass --force to overwrite it",
                output.display()
            );
        }
        fs::remove_dir_all(output)
            .with_context(|| format!("failed to remove {}", output.display()))?;
    }

    fs::create_dir_all(output.join("memory"))?;
    fs::create_dir_all(output.join("skills"))?;
    fs::create_dir_all(output.join("prompts"))?;
    fs::create_dir_all(output.join("tools"))?;
    fs::create_dir_all(output.join("meta"))?;
    Ok(())
}

fn read_first_existing(workspace: &Path, files: &[&str]) -> Result<Option<String>> {
    for file in files {
        if let Some(content) = read_optional(workspace.join(file))? {
            if !content.trim().is_empty() {
                return Ok(Some(content));
            }
        }
    }
    Ok(None)
}

fn read_named_files(workspace: &Path, files: &[&str]) -> Result<serde_json::Value> {
    let mut entries = serde_json::Map::new();
    for file in files {
        if let Some(content) = read_optional(workspace.join(file))? {
            entries.insert((*file).to_string(), serde_json::Value::String(content));
        }
    }
    Ok(serde_json::Value::Object(entries))
}

fn import_persona(platform: PlatformImportKind, instructions: &str) -> String {
    let mut lines = instructions.lines().map(str::trim).filter(|line| {
        !line.is_empty()
            && !line.eq_ignore_ascii_case("custom instructions")
            && !line.eq_ignore_ascii_case("system")
            && !line.eq_ignore_ascii_case("SOUL.md")
            && !line.starts_with("FROM ")
            && !line.starts_with("SYSTEM")
            && *line != "\"\"\""
    });
    lines
        .next()
        .unwrap_or_else(|| platform.label())
        .trim_matches('"')
        .to_string()
}

fn render_platform_import_manifest(
    name: &str,
    persona: &str,
    platform: PlatformImportKind,
) -> String {
    let (provider, model) = platform.preferred_model();
    format!(
        r#"apiVersion: agentvm/v1
kind: AgentImage
metadata:
  name: "{name}"
  version: "1.0.0"
  displayName: "{display_name}"
  description: "Imported from a {platform_label} workspace"
  tags: ["{platform_slug}", "imported"]
identity:
  name: "{display_name}"
  persona: |
{persona_block}
memory:
  strategy:
    consolidationFrequency: "manual"
    forgettingPolicy: "none"
    retrievalMethod: "local"
  episodic:
    source: "memory/episodic.md"
    format: "structured-markdown"
  semantic:
    source: "memory/semantic.json"
    format: "key-value-store"
  procedural:
    source: "memory/procedural.yaml"
    format: "skill-manifest"
  social:
    source: "memory/social.yaml"
    format: "contact-graph"
skills:
  builtin: []
tools:
  denied: []
prompts:
  system:
    source: "prompts/system.md"
  constraints:
    source: "prompts/constraints.md"
runtime:
  preferredModels:
    - provider: "{provider}"
      model: "{model}"
      priority: 1
"#,
        display_name = title_from_slug(name),
        platform_label = platform.label(),
        platform_slug = platform.slug(),
        persona_block = indent_block(persona, 4),
    )
}

fn render_platform_import_readme(
    name: &str,
    workspace: &Path,
    platform: PlatformImportKind,
) -> String {
    format!(
        "# {}\n\nImported from {} workspace `{}`.\n\nReview memory files before publishing this image.\n",
        title_from_slug(name),
        platform.label(),
        workspace.display()
    )
}

fn import_openclaw_skills(workspace: &Path, output: &Path) -> Result<Vec<SkillEntry>> {
    let skills_dir = workspace.join("skills");
    if !skills_dir.exists() {
        return Ok(Vec::new());
    }
    let mut skills = Vec::new();
    for entry in fs::read_dir(&skills_dir)
        .with_context(|| format!("failed to read {}", skills_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let id = local_skill_id(path.to_string_lossy().as_ref())?;
        let target = output.join("skills").join(&id);
        copy_dir_all(&path, &target)?;
        skills.push(SkillEntry {
            path: Some(format!("skills/{id}/")),
            id,
            version: Some("1.0.0".to_string()),
            enabled: Some(true),
            config: HashMap::new(),
        });
    }
    skills.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(skills)
}

fn copy_dir_all(source: &Path, target: &Path) -> Result<()> {
    fs::create_dir_all(target)?;
    for entry in
        fs::read_dir(source).with_context(|| format!("failed to read {}", source.display()))?
    {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let destination = target.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_all(&entry.path(), &destination)?;
        } else if file_type.is_file() {
            fs::copy(entry.path(), &destination).with_context(|| {
                format!(
                    "failed to copy {} -> {}",
                    entry.path().display(),
                    destination.display()
                )
            })?;
        }
    }
    Ok(())
}

fn read_optional(path: PathBuf) -> Result<Option<String>> {
    match fs::read_to_string(&path) {
        Ok(value) => Ok(Some(value)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error).with_context(|| format!("failed to read {}", path.display())),
    }
}

fn first_non_empty<'a>(values: &[Option<&'a str>]) -> Option<&'a str> {
    values
        .iter()
        .flatten()
        .copied()
        .find(|value| !value.trim().is_empty())
}

fn render_openclaw_import_manifest(
    name: &str,
    persona: &str,
    skill_entries: &[SkillEntry],
) -> String {
    let skills = if skill_entries.is_empty() {
        "  builtin: []\n".to_string()
    } else {
        let entries = skill_entries
            .iter()
            .map(|skill| {
                format!(
                    "    - id: {}\n      version: {}\n      path: {}\n      enabled: true\n",
                    skill.id,
                    skill.version.as_deref().unwrap_or("1.0.0"),
                    skill.path.as_deref().unwrap_or("")
                )
            })
            .collect::<String>();
        format!("  builtin:\n{entries}")
    };

    format!(
        r#"apiVersion: agentvm/v1
kind: AgentImage
metadata:
  name: "{name}"
  version: "1.0.0"
  displayName: "{display_name}"
  description: "Imported from an OpenClaw workspace"
  tags: ["openclaw", "imported"]
identity:
  name: "{display_name}"
  persona: |
{persona_block}
memory:
  strategy:
    consolidationFrequency: "manual"
    forgettingPolicy: "none"
    retrievalMethod: "local"
  episodic:
    source: "memory/episodic.md"
    format: "structured-markdown"
  semantic:
    source: "memory/semantic.json"
    format: "key-value-store"
  procedural:
    source: "memory/procedural.yaml"
    format: "skill-manifest"
  social:
    source: "memory/social.yaml"
    format: "contact-graph"
skills:
{skills}tools:
  denied: []
prompts:
  system:
    source: "prompts/system.md"
  constraints:
    source: "prompts/constraints.md"
runtime:
  preferredModels:
    - provider: "openclaw"
      model: "portable-default"
      priority: 1
"#,
        display_name = title_from_slug(name),
        persona_block = indent_block(persona, 4),
    )
}

fn render_imported_procedural(skills: &[SkillEntry]) -> String {
    let mut output = "version: 1\nskills:\n".to_string();
    if skills.is_empty() {
        output.push_str("  []\n");
        return output;
    }
    for skill in skills {
        output.push_str(&format!(
            "  - name: {}\n    learned: imported\n    confidence: 1.0\n    trigger: {}\n",
            skill.id, skill.id
        ));
    }
    output
}

fn render_import_readme(name: &str, workspace: &Path) -> String {
    format!(
        "# {}\n\nImported from OpenClaw workspace `{}`.\n\nReview memory files before publishing this image.\n",
        title_from_slug(name),
        workspace.display()
    )
}

fn slug_from_path(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(slugify)
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| "openclaw-import".to_string())
}

fn slugify(value: &str) -> String {
    value
        .trim()
        .to_ascii_lowercase()
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '-' {
                character
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

fn title_from_slug(value: &str) -> String {
    value
        .split('-')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn indent_block(value: &str, spaces: usize) -> String {
    let padding = " ".repeat(spaces);
    value
        .lines()
        .map(|line| format!("{padding}{line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn add_skill_reference(
    image: &mut AgentImage,
    reference: &str,
    default_version: &str,
    disabled: bool,
) -> Result<String> {
    if let Some(raw_id) = reference.strip_prefix("registry://skills/") {
        let (id, version) = split_skill_version(raw_id, default_version)?;
        image.skills.registry.retain(|skill| skill.id != id);
        image.skills.registry.push(RegistrySkill {
            id: id.to_string(),
            version: version.to_string(),
            source: reference.to_string(),
            installed: None,
        });
        return Ok(format!("registry skill {id}@{version}"));
    }

    let id = local_skill_id(reference)?;
    image.skills.builtin.retain(|skill| skill.id != id);
    image.skills.builtin.push(SkillEntry {
        id: id.clone(),
        version: Some(default_version.to_string()),
        path: Some(reference.to_string()),
        enabled: Some(!disabled),
        config: HashMap::new(),
    });
    Ok(format!("built-in skill {id}@{default_version}"))
}

fn split_skill_version<'a>(
    raw_id: &'a str,
    default_version: &'a str,
) -> Result<(&'a str, &'a str)> {
    let (id, version) = raw_id.split_once('@').unwrap_or((raw_id, default_version));
    if id.trim().is_empty() {
        anyhow::bail!("registry skill id cannot be empty");
    }
    if version.trim().is_empty() {
        anyhow::bail!("registry skill version cannot be empty");
    }
    Ok((id, version))
}

fn local_skill_id(reference: &str) -> Result<String> {
    let trimmed = reference.trim();
    if trimmed.is_empty() {
        anyhow::bail!("skill reference cannot be empty");
    }

    let without_scheme = trimmed.strip_prefix("builtin:").unwrap_or(trimmed);
    let normalized_path = without_scheme.trim_end_matches('/');
    let path = Path::new(normalized_path);
    let candidate = if path.file_name().and_then(|name| name.to_str()) == Some("SKILL.md") {
        path.parent()
            .and_then(|parent| parent.file_name())
            .and_then(|name| name.to_str())
            .unwrap_or(normalized_path)
    } else {
        path.file_stem()
            .and_then(|name| name.to_str())
            .or_else(|| path.file_name().and_then(|name| name.to_str()))
            .unwrap_or(normalized_path)
    };

    let id = candidate
        .to_ascii_lowercase()
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '-' {
                character
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string();

    if id.is_empty() {
        anyhow::bail!("could not derive a skill id from '{reference}'");
    }
    Ok(id)
}

fn remove_skill(image: &mut AgentImage, id: &str) -> usize {
    let builtin_before = image.skills.builtin.len();
    let registry_before = image.skills.registry.len();
    image.skills.builtin.retain(|skill| skill.id != id);
    image.skills.registry.retain(|skill| skill.id != id);
    builtin_before - image.skills.builtin.len() + registry_before - image.skills.registry.len()
}

fn bump_version(version: &str, level: BumpLevel) -> Result<String> {
    let mut parts = version
        .split('.')
        .map(|part| {
            part.parse::<u64>()
                .with_context(|| format!("invalid numeric version component '{part}'"))
        })
        .collect::<Result<Vec<_>>>()?;
    if parts.len() < 2 || parts.len() > 3 {
        anyhow::bail!("version '{version}' must be major.minor or major.minor.patch");
    }
    while parts.len() < 3 {
        parts.push(0);
    }

    match level {
        BumpLevel::Patch => parts[2] += 1,
        BumpLevel::Minor => {
            parts[1] += 1;
            parts[2] = 0;
        }
        BumpLevel::Major => {
            parts[0] += 1;
            parts[1] = 0;
            parts[2] = 0;
        }
    }

    Ok(format!("{}.{}.{}", parts[0], parts[1], parts[2]))
}

fn read_changelog(path: &Path) -> Result<String> {
    let changelog_path = resolve_changelog_path(path)?;
    match fs::read_to_string(&changelog_path) {
        Ok(value) => Ok(value),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let image = read_image(path)?;
            Ok(format!(
                "# Changelog\n\nNo CHANGELOG.md found for {} {}.\n",
                image.metadata.name, image.metadata.version
            ))
        }
        Err(error) => {
            Err(error).with_context(|| format!("failed to read {}", changelog_path.display()))
        }
    }
}

fn prepend_changelog_entry(path: &Path, version: &str, message: &str) -> Result<()> {
    let changelog_path = resolve_changelog_path(path)?;
    let existing =
        fs::read_to_string(&changelog_path).unwrap_or_else(|_| "# Changelog\n\n".to_string());
    let entry = format!("## {version}\n- {message}\n\n");
    let updated = if let Some(rest) = existing.strip_prefix("# Changelog\n\n") {
        format!("# Changelog\n\n{entry}{rest}")
    } else if let Some(rest) = existing.strip_prefix("# Changelog\n") {
        format!("# Changelog\n\n{entry}{}", rest.trim_start_matches('\n'))
    } else {
        format!("# Changelog\n\n{entry}{existing}")
    };
    fs::write(&changelog_path, updated)
        .with_context(|| format!("failed to write {}", changelog_path.display()))
}

fn resolve_changelog_path(path: &Path) -> Result<PathBuf> {
    if path.is_dir() {
        return Ok(path.join("CHANGELOG.md"));
    }
    if path.file_name().and_then(|name| name.to_str()) == Some("agent.yaml") {
        return path
            .parent()
            .map(|parent| parent.join("CHANGELOG.md"))
            .with_context(|| format!("{} has no parent directory", path.display()));
    }
    Ok(path
        .parent()
        .map(|parent| parent.join("CHANGELOG.md"))
        .unwrap_or_else(|| PathBuf::from("CHANGELOG.md")))
}

struct RegistryReference {
    raw: String,
    owner: String,
    name: String,
    version: Option<String>,
}

fn registry_get(base_url: &str, path: &str) -> Result<String> {
    registry_request(base_url, "GET", path, None)
}

fn registry_post(base_url: &str, path: &str, body: &str) -> Result<String> {
    registry_request(base_url, "POST", path, Some(body))
}

fn registry_request(
    base_url: &str,
    method: &str,
    path: &str,
    body: Option<&str>,
) -> Result<String> {
    let endpoint = parse_http_endpoint(base_url)?;
    let mut stream = TcpStream::connect(&endpoint.address)
        .with_context(|| format!("failed to connect to registry {}", endpoint.address))?;
    let body = body.unwrap_or("");
    let request = format!(
        "{method} {path} HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        endpoint.host,
        body.len(),
        body
    );
    stream.write_all(request.as_bytes())?;
    let mut response = String::new();
    stream.read_to_string(&mut response)?;
    let (head, body) = response
        .split_once("\r\n\r\n")
        .with_context(|| "invalid HTTP response from registry")?;
    let status = head
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|status| status.parse::<u16>().ok())
        .with_context(|| "invalid HTTP status from registry")?;
    if !(200..300).contains(&status) {
        anyhow::bail!("registry returned HTTP {status}: {body}");
    }
    if has_chunked_transfer_encoding(head) {
        return decode_chunked_body(body);
    }
    Ok(body.to_string())
}

fn has_chunked_transfer_encoding(head: &str) -> bool {
    head.lines().any(|line| {
        line.split_once(':')
            .map(|(name, value)| {
                name.eq_ignore_ascii_case("transfer-encoding")
                    && value.to_ascii_lowercase().contains("chunked")
            })
            .unwrap_or(false)
    })
}

fn decode_chunked_body(body: &str) -> Result<String> {
    let mut remaining = body;
    let mut decoded = String::new();
    loop {
        let (size_line, after_size) = remaining
            .split_once("\r\n")
            .with_context(|| "invalid chunked registry response")?;
        let size_hex = size_line.split(';').next().unwrap_or(size_line).trim();
        let size = usize::from_str_radix(size_hex, 16)
            .with_context(|| format!("invalid chunk size '{size_hex}'"))?;
        if size == 0 {
            return Ok(decoded);
        }
        if after_size.len() < size + 2 {
            anyhow::bail!("truncated chunked registry response");
        }
        decoded.push_str(&after_size[..size]);
        remaining = &after_size[size..];
        remaining = remaining
            .strip_prefix("\r\n")
            .with_context(|| "invalid chunk terminator in registry response")?;
    }
}

struct HttpEndpoint {
    host: String,
    address: String,
}

fn parse_http_endpoint(base_url: &str) -> Result<HttpEndpoint> {
    let without_scheme = base_url
        .strip_prefix("http://")
        .with_context(|| "only plain http:// registry URLs are supported")?;
    let host_port = without_scheme
        .split('/')
        .next()
        .filter(|value| !value.is_empty())
        .with_context(|| "registry URL is missing a host")?;
    let address = if host_port.contains(':') {
        host_port.to_string()
    } else {
        format!("{host_port}:80")
    };
    Ok(HttpEndpoint {
        host: host_port.to_string(),
        address,
    })
}

fn render_registry_images(body: &str) -> Result<String> {
    let value: serde_json::Value = serde_json::from_str(body)?;
    let images = value
        .get("images")
        .and_then(|images| images.as_array())
        .with_context(|| "registry response did not include images[]")?;
    if images.is_empty() {
        return Ok("no registry images found\n".to_string());
    }

    let mut output = format!("{} registry image(s)\n", images.len());
    for image in images {
        output.push_str(&format!(
            "{}/{}:{} {}\n",
            image["owner"].as_str().unwrap_or("unknown"),
            image["name"].as_str().unwrap_or("unknown"),
            image["version"].as_str().unwrap_or("unknown"),
            image["description"].as_str().unwrap_or("")
        ));
    }
    Ok(output)
}

fn parse_registry_reference(reference: &str) -> Result<RegistryReference> {
    let (owner, rest) = reference
        .split_once('/')
        .with_context(|| "registry reference must be owner/name or owner/name:version")?;
    if owner.trim().is_empty() || rest.trim().is_empty() {
        anyhow::bail!("registry reference must include owner and name");
    }
    let (name, version) = rest
        .split_once(':')
        .map(|(name, version)| (name, Some(version.to_string())))
        .unwrap_or((rest, None));
    if name.trim().is_empty() {
        anyhow::bail!("registry reference name cannot be empty");
    }
    Ok(RegistryReference {
        raw: reference.to_string(),
        owner: owner.to_string(),
        name: name.to_string(),
        version,
    })
}

fn url_encode(value: &str) -> String {
    let mut output = String::new();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                output.push(byte as char)
            }
            b' ' => output.push_str("%20"),
            _ => output.push_str(&format!("%{byte:02X}")),
        }
    }
    output
}

fn pack_image(directory: &Path, archive: &Path) -> Result<()> {
    let image = read_image(directory)?;
    let report = ImageValidator::new().validate(&image)?;
    if !report.valid {
        anyhow::bail!(
            "{}",
            render_validation_report(&directory.join("agent.yaml"), &report)
        );
    }

    let file = std::fs::File::create(archive)
        .with_context(|| format!("failed to create {}", archive.display()))?;
    let encoder = GzEncoder::new(file, Compression::default());
    let mut builder = tar::Builder::new(encoder);
    builder
        .append_dir_all(".", directory)
        .with_context(|| format!("failed to archive {}", directory.display()))?;
    builder.finish()?;
    Ok(())
}

fn unpack_image(archive: &Path, directory: &Path) -> Result<()> {
    if is_json_bundle_path(archive) {
        return unpack_browser_bundle(archive, directory);
    }

    std::fs::create_dir_all(directory)?;
    let file = std::fs::File::open(archive)
        .with_context(|| format!("failed to open {}", archive.display()))?;
    let decoder = GzDecoder::new(file);
    let mut archive_reader = tar::Archive::new(decoder);
    archive_reader
        .unpack(directory)
        .with_context(|| format!("failed to unpack into {}", directory.display()))?;

    let image = read_image(directory)?;
    let report = ImageValidator::new().validate(&image)?;
    if !report.valid {
        anyhow::bail!(
            "{}",
            render_validation_report(&directory.join("agent.yaml"), &report)
        );
    }
    Ok(())
}

fn unpack_browser_bundle(bundle: &Path, directory: &Path) -> Result<()> {
    let text = fs::read_to_string(bundle)
        .with_context(|| format!("failed to read {}", bundle.display()))?;
    let value: serde_json::Value = serde_json::from_str(&text)
        .with_context(|| format!("failed to parse {}", bundle.display()))?;
    let files = value
        .get("files")
        .and_then(|files| files.as_object())
        .with_context(|| format!("{} is missing a files object", bundle.display()))?;

    fs::create_dir_all(directory)
        .with_context(|| format!("failed to create {}", directory.display()))?;
    for (relative, content) in files {
        let content = content
            .as_str()
            .with_context(|| format!("bundle file {relative} is not a string"))?;
        let destination = safe_bundle_destination(directory, relative)?;
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
        fs::write(&destination, content)
            .with_context(|| format!("failed to write {}", destination.display()))?;
    }

    if !directory.join("agent.yaml").exists() {
        let image = read_image_text(&text, bundle)?;
        fs::write(directory.join("agent.yaml"), image.to_yaml()?).with_context(|| {
            format!("failed to write {}", directory.join("agent.yaml").display())
        })?;
    }

    let image = read_image(directory)?;
    let report = ImageValidator::new().validate(&image)?;
    if !report.valid {
        anyhow::bail!(
            "{}",
            render_validation_report(&directory.join("agent.yaml"), &report)
        );
    }
    Ok(())
}

fn safe_bundle_destination(directory: &Path, relative: &str) -> Result<PathBuf> {
    let relative_path = Path::new(relative);
    if relative_path.is_absolute()
        || relative_path
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        anyhow::bail!("bundle file path {relative} is not safe");
    }
    Ok(directory.join(relative_path))
}

fn is_json_bundle_path(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
}

fn export_image(image: &AgentImage, target: ExportTarget, directory: &Path) -> Result<()> {
    std::fs::create_dir_all(directory)?;

    match target {
        ExportTarget::Chatgpt => {
            std::fs::write(directory.join("custom-instructions.md"), image.persona())?;
            std::fs::write(
                directory.join("knowledge-base.md"),
                render_knowledge_base(image),
            )?;
            std::fs::write(
                directory.join("gpt-config.json"),
                serde_json::json!({
                    "name": image.display_name(),
                    "description": image.metadata.description,
                    "instructionsFile": "custom-instructions.md",
                    "knowledgeFile": "knowledge-base.md"
                })
                .to_string(),
            )?;
        }
        ExportTarget::Claude => {
            std::fs::write(directory.join("project-instructions.md"), image.persona())?;
            std::fs::write(
                directory.join("project-knowledge.md"),
                render_knowledge_base(image),
            )?;
            std::fs::create_dir_all(directory.join("skills"))?;
        }
        ExportTarget::Gemini => {
            std::fs::write(directory.join("gem-instructions.md"), image.persona())?;
            std::fs::write(
                directory.join("knowledge-bundle.md"),
                render_knowledge_base(image),
            )?;
            std::fs::write(
                directory.join("gem-config.json"),
                serde_json::to_string_pretty(&serde_json::json!({
                    "name": image.display_name(),
                    "description": image.metadata.description,
                    "instructionsFile": "gem-instructions.md",
                    "knowledgeFile": "knowledge-bundle.md"
                }))?,
            )?;
        }
        ExportTarget::Openclaw => {
            std::fs::write(directory.join("SOUL.md"), image.persona())?;
            std::fs::write(directory.join("USER.md"), render_social_memory_stub(image))?;
            std::fs::write(directory.join("AGENTS.md"), render_openclaw_agents(image))?;
            std::fs::write(directory.join("MEMORY.md"), render_knowledge_base(image))?;
            std::fs::create_dir_all(directory.join("skills"))?;
        }
        ExportTarget::Ollama => {
            std::fs::write(directory.join("Modelfile"), render_ollama_modelfile(image))?;
            std::fs::write(directory.join("system-prompt.md"), image.persona())?;
            std::fs::create_dir_all(directory.join("context"))?;
            std::fs::write(directory.join("context/agent.json"), image.to_json()?)?;
        }
    }

    Ok(())
}

fn infer_runtime_platform(image: &AgentImage) -> ExportTarget {
    image
        .preferred_model()
        .map(|model| model.provider.to_ascii_lowercase())
        .as_deref()
        .map(|provider| {
            if provider.contains("anthropic") || provider.contains("claude") {
                ExportTarget::Claude
            } else if provider.contains("gemini") || provider.contains("google") {
                ExportTarget::Gemini
            } else if provider.contains("ollama") || provider.contains("local") {
                ExportTarget::Ollama
            } else if provider.contains("openclaw") {
                ExportTarget::Openclaw
            } else {
                ExportTarget::Chatgpt
            }
        })
        .unwrap_or(ExportTarget::Chatgpt)
}

fn render_runtime_text(
    path: &Path,
    image: &AgentImage,
    platform: ExportTarget,
    prompt: &str,
) -> String {
    let store = load_memory_for_runtime(path).ok();
    let memory_documents = store.as_ref().map(MemoryStore::len).unwrap_or(0);
    let model = runtime_model_for_platform(image, platform)
        .map(|model| format!("{}/{}", model.provider, model.model))
        .unwrap_or_else(|| "platform-default".to_string());

    format!(
        "mode: dry-run\nplatform: {}\nagent: {}\nmodel: {}\nprompt: {}\n\nSYSTEM\n{}\n\nMEMORY\n{}\n\nSKILLS\n{}\n\nTOOL POLICY\n{}\n",
        export_target_slug(platform),
        image.display_name(),
        model,
        prompt,
        image.persona(),
        render_runtime_memory(store.as_ref(), memory_documents),
        render_skill_list(image),
        render_runtime_tool_policy(image)
    )
}

fn render_runtime_json(
    path: &Path,
    image: &AgentImage,
    platform: ExportTarget,
    prompt: &str,
) -> Result<String> {
    let store = load_memory_for_runtime(path).ok();
    let memories = store
        .as_ref()
        .map(|store| {
            store
                .documents()
                .iter()
                .map(|document| {
                    serde_json::json!({
                        "kind": document.kind.as_str(),
                        "id": document.id,
                        "source": document.source.display().to_string(),
                        "text": document.text
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let model = runtime_model_for_platform(image, platform).map(|model| {
        serde_json::json!({
            "provider": model.provider,
            "model": model.model,
            "priority": model.priority
        })
    });

    Ok(serde_json::to_string_pretty(&serde_json::json!({
        "mode": "dry-run",
        "platform": export_target_slug(platform),
        "agent": {
            "name": image.metadata.name,
            "displayName": image.display_name(),
            "version": image.metadata.version
        },
        "model": model,
        "turn": {
            "role": "user",
            "content": prompt
        },
        "system": image.persona(),
        "memory": memories,
        "skills": {
            "builtin": image.skills.builtin,
            "registry": image.skills.registry
        },
        "tools": {
            "denied": image.tools.denied,
            "security": image.tools.security
        }
    }))?)
}

fn runtime_model_for_platform(
    image: &AgentImage,
    platform: ExportTarget,
) -> Option<&agentvm_core::ModelPreference> {
    image
        .runtime
        .preferred_models
        .iter()
        .filter(|model| provider_matches_platform(&model.provider, platform))
        .min_by_key(|model| model.priority.unwrap_or(99))
}

fn provider_matches_platform(provider: &str, platform: ExportTarget) -> bool {
    let provider = provider.to_ascii_lowercase();
    match platform {
        ExportTarget::Chatgpt => provider.contains("openai") || provider.contains("chatgpt"),
        ExportTarget::Claude => provider.contains("anthropic") || provider.contains("claude"),
        ExportTarget::Gemini => provider.contains("gemini") || provider.contains("google"),
        ExportTarget::Openclaw => provider.contains("openclaw"),
        ExportTarget::Ollama => provider.contains("ollama") || provider.contains("local"),
    }
}

fn load_memory_for_runtime(path: &Path) -> Result<MemoryStore> {
    let image_dir = resolve_image_dir(path)?;
    MemoryStore::load(&image_dir)
}

fn render_runtime_memory(store: Option<&MemoryStore>, memory_documents: usize) -> String {
    let Some(store) = store else {
        return "- No local memory directory loaded.\n".to_string();
    };
    if store.is_empty() {
        return "- No memory documents found.\n".to_string();
    }

    let mut output = format!("- {memory_documents} memory document(s) loaded.\n");
    for document in store.documents().iter().take(4) {
        output.push_str(&format!(
            "- {} {} from {}\n",
            document.kind.as_str(),
            document.id,
            document.source.display()
        ));
    }
    output
}

fn render_runtime_tool_policy(image: &AgentImage) -> String {
    let denied = render_denied_tool_list(image);
    let security = image
        .tools
        .security
        .as_ref()
        .map(|security| {
            format!(
                "execPolicy={:?} networkPolicy={:?}",
                security.exec_policy, security.network_policy
            )
        })
        .unwrap_or_else(|| "default security policy".to_string());
    format!("{denied}\n{security}")
}

fn default_archive_path(directory: &Path) -> PathBuf {
    let name = directory
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("agent");
    PathBuf::from(format!("{name}.agentvm"))
}

fn default_unpack_path(archive: &Path) -> PathBuf {
    let file_name = archive
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("agent.agentvm");
    PathBuf::from(
        file_name
            .trim_end_matches(".agentvm.json")
            .trim_end_matches(".agentvm"),
    )
}

fn validation_exit_code(report: &ValidationReport, strict: bool) -> i32 {
    if report.valid && (!strict || report.warnings.is_empty()) {
        0
    } else {
        1
    }
}

fn render_validation_report(path: &Path, report: &ValidationReport) -> String {
    let status = if report.valid { "valid" } else { "invalid" };
    let mut output = format!("{}: {status}\n", path.display());

    for error in &report.errors {
        output.push_str(&format!("error {}: {}\n", error.field, error.message));
    }

    for warning in &report.warnings {
        output.push_str(&format!("warning {}: {}\n", warning.field, warning.message));
    }

    output
}

fn render_image_summary(image: &AgentImage) -> String {
    let model = image
        .preferred_model()
        .map(|model| format!("{}/{}", model.provider, model.model))
        .unwrap_or_else(|| "none".to_string());

    format!(
        "name: {}\ndisplay: {}\nversion: {}\nkind: {}\npersona: {}\nskills: {}\ndenied_tools: {}\npreferred_model: {}\n",
        image.metadata.name,
        image.display_name(),
        image.metadata.version,
        image.kind,
        image.persona(),
        image.skills.builtin.len(),
        image.tools.denied.len(),
        model
    )
}

fn render_diff(image_diff: &ImageDiff) -> String {
    if image_diff.identical {
        return "images are identical\n".to_string();
    }

    let mut output = format!("{} change(s)\n", image_diff.changes.len());
    for change in &image_diff.changes {
        output.push_str(&format!(
            "{:?} {}: {:?} {:?} -> {:?}\n",
            change.category, change.field, change.diff_type, change.old_value, change.new_value
        ));
    }
    output
}

fn render_memory_list(store: &MemoryStore) -> String {
    if store.is_empty() {
        return "no memory documents found\n".to_string();
    }

    let mut output = format!("{} memory document(s)\n", store.len());
    for document in store.documents() {
        output.push_str(&format!(
            "{} {} ({})\n",
            document.kind.as_str(),
            document.id,
            document.source.display()
        ));
    }
    output
}

fn render_skills(image: &AgentImage) -> String {
    let total = image.skills.builtin.len() + image.skills.registry.len();
    if total == 0 {
        return "no skills declared\n".to_string();
    }

    let mut output = format!("{total} skill reference(s)\n");
    for skill in &image.skills.builtin {
        output.push_str(&format!(
            "builtin {} {} enabled={}\n",
            skill.id,
            skill.version.as_deref().unwrap_or("unspecified"),
            skill.enabled.unwrap_or(true)
        ));
    }
    for skill in &image.skills.registry {
        output.push_str(&format!(
            "registry {} {} {}\n",
            skill.id, skill.version, skill.source
        ));
    }
    output
}

fn render_search_hits(hits: &[agentvm_memory::SearchHit]) -> String {
    if hits.is_empty() {
        return "no matching memory found\n".to_string();
    }

    let mut output = format!("{} result(s)\n", hits.len());
    for hit in hits {
        output.push_str(&format!(
            "{:.3} {} {}\n{}\n\n",
            hit.score,
            hit.document.kind.as_str(),
            hit.document.id,
            hit.document.text
        ));
    }
    output
}

fn render_consolidation_report(store: &MemoryStore) -> String {
    let report = consolidate(store);
    let mut output = format!(
        "documents: {}\nestimated_bytes: {}\n",
        report.total_documents, report.estimated_bytes
    );

    output.push_str("by_kind:\n");
    for (kind, count) in report.documents_by_kind {
        output.push_str(&format!("  {kind}: {count}\n"));
    }

    output.push_str("top_terms:\n");
    for (term, count) in report.top_terms {
        output.push_str(&format!("  {term}: {count}\n"));
    }

    output
}

fn render_knowledge_base(image: &AgentImage) -> String {
    format!(
        "# {}\n\n{}\n\n## Persona\n\n{}\n\n## Skills\n\n{}\n\n## Denied Tools\n\n{}\n",
        image.display_name(),
        image
            .metadata
            .description
            .as_deref()
            .unwrap_or("Portable AgentVM image."),
        image.persona(),
        render_skill_list(image),
        render_denied_tool_list(image)
    )
}

fn render_skill_list(image: &AgentImage) -> String {
    if image.skills.builtin.is_empty() && image.skills.registry.is_empty() {
        return "- No skills declared.\n".to_string();
    }

    let mut lines = image
        .skills
        .builtin
        .iter()
        .map(|skill| format!("- {} (built-in)", skill.id))
        .collect::<Vec<_>>();
    lines.extend(
        image
            .skills
            .registry
            .iter()
            .map(|skill| format!("- {}@{} ({})", skill.id, skill.version, skill.source)),
    );
    lines.join("\n")
}

fn render_denied_tool_list(image: &AgentImage) -> String {
    if image.tools.denied.is_empty() {
        return "- No denied tools declared.\n".to_string();
    }

    image
        .tools
        .denied
        .iter()
        .map(|tool| format!("- {tool}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_social_memory_stub(image: &AgentImage) -> String {
    format!(
        "# User and Social Memory\n\nExported from AgentVM image `{}`.\n\nReview `MEMORY.md` before importing sensitive memories into another runtime.\n",
        image.metadata.name
    )
}

fn render_openclaw_agents(image: &AgentImage) -> String {
    format!(
        "# {}\n\n{}\n\nFollow the AgentVM exported memory files and preserve user-owned context portability.\n",
        image.display_name(),
        image.persona()
    )
}

fn render_ollama_modelfile(image: &AgentImage) -> String {
    format!(
        "FROM llama3.2\n\nSYSTEM \"\"\"\n{}\n\"\"\"\n",
        image.persona().replace("\"\"\"", "\\\"\\\"\\\"")
    )
}

fn export_target_slug(target: ExportTarget) -> &'static str {
    match target {
        ExportTarget::Chatgpt => "chatgpt",
        ExportTarget::Claude => "claude",
        ExportTarget::Gemini => "gemini",
        ExportTarget::Openclaw => "openclaw",
        ExportTarget::Ollama => "ollama",
    }
}

struct TemplateManifest {
    agent_yaml: &'static str,
    readme: &'static str,
    system_prompt: &'static str,
}

fn template_manifest(template: TemplateKind) -> TemplateManifest {
    match template {
        TemplateKind::SeniorDev => TemplateManifest {
            agent_yaml: include_str!("templates/senior-dev.yaml"),
            readme: "# Senior Dev Agent\n\nPractical, security-conscious software engineering assistant.\n",
            system_prompt: "You are a senior developer. Prefer scoped changes, tests, and direct evidence.\n",
        },
        TemplateKind::CreativeWriter => TemplateManifest {
            agent_yaml: include_str!("templates/creative-writer.yaml"),
            readme: "# Creative Writer Agent\n\nStory-focused assistant for narrative work.\n",
            system_prompt: "You are a creative writing partner focused on structure, voice, and vivid revision.\n",
        },
        TemplateKind::Researcher => TemplateManifest {
            agent_yaml: include_str!("templates/researcher.yaml"),
            readme: "# Researcher Agent\n\nEvidence-first assistant for analysis and synthesis.\n",
            system_prompt: "You are a meticulous researcher. Cite sources and separate evidence from inference.\n",
        },
        TemplateKind::CustomerSupport => TemplateManifest {
            agent_yaml: include_str!("templates/customer-support.yaml"),
            readme: "# Customer Support Agent\n\nPatient support assistant for triage and escalation.\n",
            system_prompt: "You are a calm support agent. Be empathetic, precise, and escalate when uncertain.\n",
        },
        TemplateKind::DataAnalyst => TemplateManifest {
            agent_yaml: include_str!("templates/data-analyst.yaml"),
            readme: "# Data Analyst Agent\n\nMetrics-focused assistant for data questions and visualization.\n",
            system_prompt: "You are a data analyst. Check assumptions and communicate with numbers and charts.\n",
        },
        TemplateKind::TurkishDev => TemplateManifest {
            agent_yaml: include_str!("templates/turkish-dev.yaml"),
            readme: "# Turkish Dev Agent\n\nTurkish-speaking senior developer assistant.\n",
            system_prompt: "Kidemli bir Turk yazilim gelistiricisisin. Kisa, dogrudan ve kanit odakli calis.\n",
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn image(name: &str, persona: &str) -> AgentImage {
        AgentImage::from_yaml(&format!(
            r#"
apiVersion: agentvm/v1
kind: AgentImage
metadata:
  name: "{name}"
  version: "1.0.0"
identity:
  persona: "{persona}"
"#
        ))
        .unwrap()
    }

    #[test]
    fn validation_exit_code_allows_warnings_without_strict() {
        let image = AgentImage::from_yaml(
            r#"
apiVersion: agentvm/v1
kind: AgentImage
metadata:
  name: "test-agent"
  version: "1.0.0"
"#,
        )
        .unwrap();
        let report = ImageValidator::new().validate(&image).unwrap();

        assert_eq!(validation_exit_code(&report, false), 0);
        assert_eq!(validation_exit_code(&report, true), 1);
    }

    #[test]
    fn image_summary_contains_core_fields() {
        let output = render_image_summary(&image("agent-one", "A focused helper"));

        assert!(output.contains("name: agent-one"));
        assert!(output.contains("persona: A focused helper"));
        assert!(output.contains("preferred_model: none"));
    }

    #[test]
    fn diff_render_reports_changed_fields() {
        let output = render_diff(&diff(
            &image("agent-one", "Old helper"),
            &image("agent-two", "New helper"),
        ));

        assert!(output.contains("change(s)"));
        assert!(output.contains("metadata.name"));
        assert!(output.contains("identity.persona"));
    }

    #[test]
    fn adding_registry_skill_replaces_existing_reference() {
        let mut image = image("agent-one", "A focused helper");

        add_skill_reference(
            &mut image,
            "registry://skills/github-advanced@3.1.0",
            "1.0.0",
            false,
        )
        .unwrap();
        add_skill_reference(
            &mut image,
            "registry://skills/github-advanced@3.2.0",
            "1.0.0",
            false,
        )
        .unwrap();

        assert_eq!(image.skills.registry.len(), 1);
        assert_eq!(image.skills.registry[0].id, "github-advanced");
        assert_eq!(image.skills.registry[0].version, "3.2.0");
    }

    #[test]
    fn adding_local_skill_derives_portable_id() {
        let mut image = image("agent-one", "A focused helper");

        add_skill_reference(&mut image, "./skills/code-review/SKILL.md", "1.2.0", true).unwrap();

        assert_eq!(image.skills.builtin.len(), 1);
        assert_eq!(image.skills.builtin[0].id, "code-review");
        assert_eq!(image.skills.builtin[0].version.as_deref(), Some("1.2.0"));
        assert_eq!(image.skills.builtin[0].enabled, Some(false));
    }

    #[test]
    fn removing_skill_checks_both_skill_sources() {
        let mut image = image("agent-one", "A focused helper");
        add_skill_reference(&mut image, "builtin:code-review", "1.0.0", false).unwrap();
        add_skill_reference(&mut image, "registry://skills/code-review", "1.0.0", false).unwrap();

        assert_eq!(remove_skill(&mut image, "code-review"), 2);
        assert!(image.skills.builtin.is_empty());
        assert!(image.skills.registry.is_empty());
    }

    #[test]
    fn runtime_text_includes_agent_context_without_provider_call() {
        let mut image = image("agent-one", "A focused helper");
        add_skill_reference(&mut image, "builtin:code-review", "1.0.0", false).unwrap();

        let output = render_runtime_text(
            Path::new("examples/minimal-agent.yaml"),
            &image,
            ExportTarget::Chatgpt,
            "Review this patch",
        );

        assert!(output.contains("mode: dry-run"));
        assert!(output.contains("platform: chatgpt"));
        assert!(output.contains("prompt: Review this patch"));
        assert!(output.contains("A focused helper"));
        assert!(output.contains("code-review"));
    }

    #[test]
    fn security_scan_passes_clean_image() {
        let root = std::env::temp_dir().join(format!(
            "agentvm-security-clean-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        init_image(&root, TemplateKind::SeniorDev, false).unwrap();

        let findings = scan_image_security(&root).unwrap();

        assert!(findings.is_empty());
        assert_eq!(security_scan_exit_code(&findings, false), 0);
        assert_eq!(security_scan_exit_code(&findings, true), 0);

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn security_scan_reports_secret_like_memory() {
        let root = std::env::temp_dir().join(format!(
            "agentvm-security-finding-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        init_image(&root, TemplateKind::SeniorDev, false).unwrap();
        std::fs::write(
            root.join("memory/episodic.md"),
            "# Episodic Memory\n\napi_key: sk-test-secret-token-value\n",
        )
        .unwrap();

        let findings = scan_image_security(&root).unwrap();
        let output = render_security_scan(&root, &findings);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].file, "memory/episodic.md");
        assert!(output.contains("openai-key"));
        assert_eq!(security_scan_exit_code(&findings, false), 0);
        assert_eq!(security_scan_exit_code(&findings, true), 1);

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn imports_openclaw_workspace_into_valid_agent_image() {
        let root = std::env::temp_dir().join(format!(
            "agentvm-openclaw-import-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let workspace = root.join("openclaw-workspace");
        let output = root.join("agentvm-image");
        std::fs::create_dir_all(workspace.join("skills/code-review")).unwrap();
        std::fs::write(workspace.join("SOUL.md"), "A focused imported helper").unwrap();
        std::fs::write(
            workspace.join("AGENTS.md"),
            "Follow local project instructions.",
        )
        .unwrap();
        std::fs::write(workspace.join("MEMORY.md"), "User prefers direct evidence.").unwrap();
        std::fs::write(workspace.join("USER.md"), "User context here.").unwrap();
        std::fs::write(
            workspace.join("skills/code-review/SKILL.md"),
            "# Code Review\n\nReview code carefully.",
        )
        .unwrap();

        import_openclaw_workspace(&workspace, &output, false).unwrap();
        let image = read_image(&output).unwrap();

        assert_eq!(image.metadata.name, "openclaw-workspace");
        assert_eq!(image.persona().trim(), "A focused imported helper");
        assert_eq!(image.skills.builtin.len(), 1);
        assert_eq!(image.skills.builtin[0].id, "code-review");
        assert_eq!(image.runtime.preferred_models.len(), 1);
        assert_eq!(image.runtime.preferred_models[0].provider, "openclaw");
        assert_eq!(image.runtime.preferred_models[0].model, "portable-default");
        assert!(output.join("skills/code-review/SKILL.md").exists());
        assert!(output.join("memory/semantic.json").exists());

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn imports_platform_workspaces_into_valid_agent_images() {
        let cases = [
            (
                PlatformImportKind::Chatgpt,
                "chatgpt-workspace",
                "custom-instructions.md",
                "A focused ChatGPT helper",
            ),
            (
                PlatformImportKind::Claude,
                "claude-workspace",
                "project-instructions.md",
                "A focused Claude helper",
            ),
            (
                PlatformImportKind::Gemini,
                "gemini-workspace",
                "gem-instructions.md",
                "A focused Gemini helper",
            ),
            (
                PlatformImportKind::Ollama,
                "ollama-workspace",
                "system-prompt.md",
                "A focused Ollama helper",
            ),
        ];

        for (platform, workspace_name, instruction_file, persona) in cases {
            let root = std::env::temp_dir().join(format!(
                "agentvm-platform-import-test-{}-{}",
                workspace_name,
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            let workspace = root.join(workspace_name);
            let output = root.join("agentvm-image");
            std::fs::create_dir_all(&workspace).unwrap();
            std::fs::write(workspace.join(instruction_file), persona).unwrap();
            if matches!(platform, PlatformImportKind::Chatgpt) {
                std::fs::write(workspace.join("knowledge-base.md"), "User prefers proof.").unwrap();
            }
            if matches!(platform, PlatformImportKind::Claude) {
                std::fs::write(
                    workspace.join("project-knowledge.md"),
                    "User prefers proof.",
                )
                .unwrap();
            }
            if matches!(platform, PlatformImportKind::Gemini) {
                std::fs::write(workspace.join("knowledge-bundle.md"), "User prefers proof.")
                    .unwrap();
                std::fs::write(workspace.join("gem-config.json"), "{}").unwrap();
            }
            if matches!(platform, PlatformImportKind::Ollama) {
                std::fs::create_dir_all(workspace.join("context")).unwrap();
                std::fs::write(workspace.join("context/agent.json"), "{}").unwrap();
            }

            import_platform_workspace(&workspace, &output, false, platform).unwrap();
            let image = read_image(&output).unwrap();

            assert_eq!(image.metadata.name, workspace_name);
            assert_eq!(image.persona().trim(), persona);
            assert!(output.join("prompts/system.md").exists());
            assert!(output.join("memory/semantic.json").exists());

            std::fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn unpacks_browser_bundle_into_valid_image_directory() {
        let root = std::env::temp_dir().join(format!(
            "agentvm-browser-bundle-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let bundle = root.join("registry-pull.json");
        let output = root.join("unpacked");
        std::fs::create_dir_all(&root).unwrap();
        let agent_yaml = r#"apiVersion: agentvm/v1
kind: AgentImage
metadata:
  name: studio-agent
  version: 1.0.0
identity:
  persona: Studio packaged helper
"#;
        let payload = serde_json::json!({
            "package": {
                "format": "agentvm-browser-bundle",
                "version": "0.1.0",
                "entrypoint": "agent.yaml"
            },
            "manifest": AgentImage::from_yaml(agent_yaml).unwrap(),
            "files": {
                "agent.yaml": agent_yaml,
                "memory/episodic.md": "# Episodic Memory\n",
                "meta/package.json": "{\"format\":\"agentvm-browser-bundle\"}\n"
            }
        });
        std::fs::write(&bundle, serde_json::to_string_pretty(&payload).unwrap()).unwrap();

        unpack_image(&bundle, &output).unwrap();

        let image = read_image(&output).unwrap();
        assert_eq!(image.metadata.name, "studio-agent");
        assert!(output.join("memory/episodic.md").exists());
        assert!(output.join("meta/package.json").exists());

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn registry_payload_contains_manifest_and_files() {
        let root = std::env::temp_dir().join(format!(
            "agentvm-registry-payload-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(root.join("memory")).unwrap();
        std::fs::write(
            root.join("agent.yaml"),
            r#"apiVersion: agentvm/v1
kind: AgentImage
metadata:
  name: registry-agent
  version: 1.0.0
identity:
  persona: Registry payload helper
"#,
        )
        .unwrap();
        std::fs::write(root.join("memory/episodic.md"), "# Episodic Memory\n").unwrap();

        let payload = registry_image_payload(&root, "local").unwrap();

        assert_eq!(payload["owner"], "local");
        assert_eq!(payload["manifest"]["metadata"]["name"], "registry-agent");
        assert!(payload["files"]["agent.yaml"]
            .as_str()
            .unwrap()
            .contains("registry-agent"));
        assert_eq!(
            payload["files"]["memory/episodic.md"].as_str().unwrap(),
            "# Episodic Memory\n"
        );

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn browser_bundle_rejects_unsafe_paths() {
        let root = std::env::temp_dir().join(format!(
            "agentvm-browser-bundle-unsafe-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let bundle = root.join("unsafe.agentvm.json");
        let output = root.join("unpacked");
        std::fs::create_dir_all(&root).unwrap();
        let payload = serde_json::json!({
            "files": {
                "../escape.txt": "nope"
            }
        });
        std::fs::write(&bundle, serde_json::to_string_pretty(&payload).unwrap()).unwrap();

        let error = unpack_image(&bundle, &output).unwrap_err().to_string();

        assert!(error.contains("not safe"));
        assert!(!root.join("escape.txt").exists());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn bump_version_updates_expected_component() {
        assert_eq!(bump_version("1.2.3", BumpLevel::Patch).unwrap(), "1.2.4");
        assert_eq!(bump_version("1.2.3", BumpLevel::Minor).unwrap(), "1.3.0");
        assert_eq!(bump_version("1.2.3", BumpLevel::Major).unwrap(), "2.0.0");
        assert_eq!(bump_version("1.2", BumpLevel::Patch).unwrap(), "1.2.1");
    }

    #[test]
    fn changelog_entry_is_prepended_after_header() {
        let root = std::env::temp_dir().join(format!(
            "agentvm-changelog-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(
            root.join("CHANGELOG.md"),
            "# Changelog\n\n## 1.0.0\n- Old\n",
        )
        .unwrap();

        prepend_changelog_entry(&root, "1.0.1", "New entry").unwrap();
        let changelog = std::fs::read_to_string(root.join("CHANGELOG.md")).unwrap();

        assert!(changelog.starts_with("# Changelog\n\n## 1.0.1\n- New entry\n\n## 1.0.0"));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn parses_registry_reference_with_optional_version() {
        let with_version = parse_registry_reference("local/senior-dev:1.2.3").unwrap();
        assert_eq!(with_version.owner, "local");
        assert_eq!(with_version.name, "senior-dev");
        assert_eq!(with_version.version.as_deref(), Some("1.2.3"));

        let without_version = parse_registry_reference("agentvm/reviewer").unwrap();
        assert_eq!(without_version.owner, "agentvm");
        assert_eq!(without_version.name, "reviewer");
        assert_eq!(without_version.version, None);
    }

    #[test]
    fn url_encode_preserves_safe_chars() {
        assert_eq!(url_encode("coding assistant"), "coding%20assistant");
        assert_eq!(url_encode("a/b?c"), "a%2Fb%3Fc");
    }

    #[test]
    fn decodes_chunked_registry_responses() {
        let body = "7\r\n{\"ok\":1\r\n2\r\n}\n\r\n0\r\n\r\n";

        assert_eq!(decode_chunked_body(body).unwrap(), "{\"ok\":1}\n");
    }
}
