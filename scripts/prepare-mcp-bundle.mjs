#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(__dirname, "..");

const sourceDir = path.join(root, "mcp-server", "dist");
const sourcePackageJson = path.join(root, "mcp-server", "package.json");
const targetDir = path.join(root, "tama-tauri", "src-tauri", "mcp-server-dist");

function copyDirContents(source, target) {
  const entries = fs.readdirSync(source, { withFileTypes: true });

  for (const entry of entries) {
    const sourcePath = path.join(source, entry.name);
    const targetPath = path.join(target, entry.name);

    if (entry.isDirectory()) {
      fs.mkdirSync(targetPath, { recursive: true });
      copyDirContents(sourcePath, targetPath);
      continue;
    }

    fs.copyFileSync(sourcePath, targetPath);
  }
}

if (!fs.existsSync(sourceDir)) {
  console.error("[prepare-mcp-bundle] missing mcp-server/dist. Run `npm --prefix mcp-server run build` first.");
  process.exit(1);
}

if (!fs.existsSync(path.join(sourceDir, "index.js"))) {
  console.error("[prepare-mcp-bundle] missing mcp-server/dist/index.js after build.");
  process.exit(1);
}

fs.rmSync(targetDir, { force: true, recursive: true });
fs.mkdirSync(targetDir, { recursive: true });

copyDirContents(sourceDir, targetDir);
fs.copyFileSync(sourcePackageJson, path.join(targetDir, "package.json"));
fs.writeFileSync(path.join(targetDir, ".gitkeep"), "\n");

console.log(`[prepare-mcp-bundle] staged MCP bundle resources in ${path.relative(root, targetDir)}`);
