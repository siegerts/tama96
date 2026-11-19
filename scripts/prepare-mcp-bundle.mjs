#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(__dirname, "..");

const sourceDir = path.join(root, "mcp-server", "dist");
const sourcePackageJson = path.join(root, "mcp-server", "package.json");
const targetDir = path.join(root, "tama-tauri", "src-tauri", "mcp-server-dist");

if (!fs.existsSync(path.join(sourceDir, "index.js"))) {
  console.error("[prepare-mcp-bundle] missing mcp-server/dist/index.js after build.");
  process.exit(1);
}

// Resolve from the MCP package so release builds need only its locked dependencies.
const { build } = createRequire(sourcePackageJson)("esbuild");
const bundle = await build({
  entryPoints: [path.join(sourceDir, "index.js")],
  bundle: true,
  platform: "node",
  target: "node18",
  format: "esm",
  banner: { js: 'import { createRequire as createNodeRequire } from "node:module"; const require = createNodeRequire(import.meta.url);' },
  legalComments: "eof",
  write: false,
});

fs.rmSync(targetDir, { force: true, recursive: true });
fs.mkdirSync(targetDir, { recursive: true });

fs.writeFileSync(path.join(targetDir, "index.js"), bundle.outputFiles[0].contents);
fs.copyFileSync(sourcePackageJson, path.join(targetDir, "package.json"));
fs.writeFileSync(path.join(targetDir, ".gitkeep"), "\n");

console.log(`[prepare-mcp-bundle] staged MCP bundle resources in ${path.relative(root, targetDir)}`);
