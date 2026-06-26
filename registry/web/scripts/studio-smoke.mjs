import { readFile } from "node:fs/promises";

import { chromium } from "playwright";
import { createServer } from "vite";

const server = await createServer({
  appType: "spa",
  logLevel: "error",
  server: {
    host: "127.0.0.1",
    port: 0,
    strictPort: false,
  },
});

let browser;

try {
  await server.listen();
  const address = server.httpServer?.address();
  if (!address || typeof address === "string") {
    throw new Error("Vite did not expose a TCP address");
  }
  const baseUrl = `http://127.0.0.1:${address.port}`;

  browser = await chromium.launch({ headless: true });
  const page = await browser.newPage({
    acceptDownloads: true,
    viewport: { width: 1440, height: 1100 },
  });
  const consoleErrors = [];
  page.on("console", (message) => {
    if (message.type() === "error") consoleErrors.push(message.text());
  });

  await page.goto(baseUrl, { waitUntil: "networkidle" });
  await page.evaluate(() => localStorage.removeItem("agentvm.studio.workspace.v1"));
  await page.reload({ waitUntil: "networkidle" });

  await page.locator(".platformImportTabs button", { hasText: "OpenClaw" }).click();
  await page.locator(".platformImportControls input").fill("Smoke OpenClaw Agent");
  await page
    .locator(".platformImportEditors textarea")
    .nth(0)
    .fill(
      "SOUL.md\n\nYou are a portable owner-controlled agent. Preserve memory, skills, tool preferences, and exact repo habits across platforms.",
    );
  await page
    .locator(".platformImportEditors textarea")
    .nth(1)
    .fill("Owner prefers exact command proof, compact status, and no platform lock-in.");
  await page.getByRole("button", { name: "Create AgentVM Image" }).click();

  await page.getByRole("button", { name: "Manifest" }).click();
  const manifest = await page.getByLabel("Agent Image manifest editor").inputValue();
  assertIncludes(manifest, "name: smoke-openclaw-agent", "manifest name");
  assertIncludes(manifest, "- openclaw", "manifest source tag");

  await page.getByRole("button", { name: "Episodic", exact: true }).click();
  let memory = await page.getByLabel("Memory file editor").inputValue();
  assertIncludes(memory, "compact status", "imported memory note");

  await page.getByLabel("Memory file editor").fill("# Episodic Memory\n\n- Studio smoke autosave survives reload.\n");
  await page.waitForTimeout(250);
  await page.reload({ waitUntil: "networkidle" });
  await page.getByRole("button", { name: "Episodic", exact: true }).click();
  memory = await page.getByLabel("Memory file editor").inputValue();
  assertIncludes(memory, "autosave survives reload", "autosaved memory");

  await page.getByRole("button", { name: "Core Memory" }).click();
  await page.getByLabel("New package file").fill("memory/customer-notes.md");
  await page.getByLabel("Template").selectOption("memory");
  await page.getByRole("button", { name: "Add file" }).click();
  await page.locator(".memoryFile", { hasText: "memory/customer-notes.md" }).click();
  const customMemory = await page.getByLabel("Memory file editor").inputValue();
  assertIncludes(customMemory, "Add portable context", "custom memory template");
  await page
    .getByLabel("Memory file editor")
    .fill("# Customer Notes\n\n- Custom package files travel with the browser bundle.\n");

  const downloadPromise = page.waitForEvent("download");
  await page.getByRole("button", { name: "Package Brain" }).click();
  const download = await downloadPromise;
  const downloadPath = await download.path();
  if (!downloadPath) throw new Error("Package Brain did not produce a downloadable file");
  const bundle = JSON.parse(await readFile(downloadPath, "utf8"));
  assertIncludes(
    bundle.files?.["prompts/imported-instructions.md"],
    "portable owner-controlled agent",
    "packaged imported instructions",
  );
  assertIncludes(bundle.files?.["meta/import-source.json"], "openclaw", "packaged import metadata");
  assertIncludes(
    bundle.files?.["memory/customer-notes.md"],
    "Custom package files travel",
    "packaged custom memory file",
  );

  await page
    .getByLabel("Memory file editor")
    .fill("# Episodic Memory\n\napi_key: sk-test-secret-token-value\n");
  await page.waitForTimeout(250);
  const safetyText = await page.locator(".inspectorCard", { hasText: "Safety Scan" }).innerText();
  assertIncludes(safetyText, "1 secret-like finding", "safety finding count");
  assertIncludes(safetyText, "openai-key", "safety finding rule");
  const packageButton = page.getByRole("button", { name: "Package Brain" });
  if (!(await packageButton.isDisabled())) {
    throw new Error("Package Brain should be disabled while Safety Scan has findings");
  }

  await page.getByRole("button", { name: "Delete selected" }).click();
  await page.waitForTimeout(250);
  const clearedSafetyText = await page.locator(".inspectorCard", { hasText: "Safety Scan" }).innerText();
  assertIncludes(clearedSafetyText, "No secret-like content found", "safety finding cleared after deleting file");
  if (await packageButton.isDisabled()) {
    throw new Error("Package Brain should be enabled after deleting the unsafe custom file");
  }

  await page.getByRole("button", { name: "Reset Workspace" }).click();
  await page.waitForTimeout(250);
  const currentTitle = await page.locator(".brainSummary h1").innerText();
  assertIncludes(currentTitle, "Turkish Dev", "reset workspace title");

  await page.locator(".platformImportTabs button", { hasText: "Gemini" }).click();
  await page.locator(".platformImportControls input").fill("Smoke Gemini Agent");
  await page
    .locator(".platformImportEditors textarea")
    .nth(0)
    .fill("Gem instructions\n\nYou preserve research memory, owner preferences, and portable context files.");
  await page
    .locator(".platformImportEditors textarea")
    .nth(1)
    .fill("Owner wants Gemini migrations to keep memory and runtime targets.");
  await page.getByRole("button", { name: "Create AgentVM Image" }).click();
  await page.getByRole("button", { name: "Manifest" }).click();
  const geminiManifest = await page.getByLabel("Agent Image manifest editor").inputValue();
  assertIncludes(geminiManifest, "name: smoke-gemini-agent", "Gemini manifest name");
  assertIncludes(geminiManifest, "- gemini", "Gemini manifest source tag");
  assertIncludes(geminiManifest, "provider: gemini", "Gemini runtime provider");

  if (consoleErrors.length > 0) {
    throw new Error(`Unexpected browser console errors:\n${consoleErrors.join("\n")}`);
  }

  console.log("studio smoke passed");
} finally {
  if (browser) await browser.close();
  await server.close();
}

function assertIncludes(value, expected, label) {
  if (typeof value !== "string" || !value.includes(expected)) {
    throw new Error(`${label} did not include ${JSON.stringify(expected)}`);
  }
}
