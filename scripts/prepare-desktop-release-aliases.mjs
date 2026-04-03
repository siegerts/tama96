#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(__dirname, "..");

const target = process.argv[2]?.trim();

if (!target) {
  console.error("Usage: node scripts/prepare-desktop-release-aliases.mjs <target-triple>");
  process.exit(1);
}

const outputRoot = path.join(root, "release-aliases", target);

const aliasPlan = {
  "aarch64-apple-darwin": [
    { pattern: /\.dmg$/i, alias: "tama96_aarch64.dmg" },
  ],
  "x86_64-apple-darwin": [
    { pattern: /\.dmg$/i, alias: "tama96_x64.dmg" },
  ],
  "x86_64-unknown-linux-gnu": [
    { pattern: /\.deb$/i, alias: "tama96_amd64.deb" },
    { pattern: /\.AppImage$/i, alias: "tama96_amd64.AppImage" },
  ],
  "x86_64-pc-windows-msvc": [
    { pattern: /-setup\.exe$/i, alias: "tama96_x64-setup.exe" },
    { pattern: /_x64_en-US\.msi$/i, alias: "tama96_x64_en-US.msi" },
  ],
};

function listFiles(dirPath) {
  const entries = fs.readdirSync(dirPath, { withFileTypes: true });
  const files = [];

  for (const entry of entries) {
    const fullPath = path.join(dirPath, entry.name);
    if (entry.isDirectory()) {
      files.push(...listFiles(fullPath));
      continue;
    }
    files.push(fullPath);
  }

  return files;
}

function resolveBundleRoot(targetTriple) {
  const candidateTargetRoots = [
    process.env.CARGO_TARGET_DIR ? path.resolve(root, process.env.CARGO_TARGET_DIR) : null,
    path.join(root, "target"),
    path.join(root, "tama-tauri", "target"),
    path.join(root, "tama-tauri", "src-tauri", "target"),
  ].filter(Boolean);

  for (const targetRoot of candidateTargetRoots) {
    const bundleRoot = path.join(targetRoot, targetTriple, "release", "bundle");
    if (fs.existsSync(bundleRoot)) {
      return bundleRoot;
    }
  }

  return path.join(candidateTargetRoots[0], targetTriple, "release", "bundle");
}

const plan = aliasPlan[target];

if (!plan) {
  console.error(`[prepare-desktop-release-aliases] unsupported target: ${target}`);
  process.exit(1);
}

const bundleRoot = resolveBundleRoot(target);

if (!fs.existsSync(bundleRoot)) {
  console.error(`[prepare-desktop-release-aliases] bundle directory not found: ${bundleRoot}`);
  process.exit(1);
}

const bundleFiles = listFiles(bundleRoot);

fs.rmSync(outputRoot, { force: true, recursive: true });
fs.mkdirSync(outputRoot, { recursive: true });

for (const { pattern, alias } of plan) {
  const match = bundleFiles.find((filePath) => pattern.test(path.basename(filePath)));
  if (!match) {
    console.error(`[prepare-desktop-release-aliases] missing bundle matching ${pattern} for ${target}`);
    process.exit(1);
  }

  const destination = path.join(outputRoot, alias);
  fs.copyFileSync(match, destination);
  console.log(`[prepare-desktop-release-aliases] ${path.relative(root, match)} -> ${path.relative(root, destination)}`);
}
