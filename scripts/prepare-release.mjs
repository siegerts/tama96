#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(__dirname, "..");
const npmCmd = process.platform === "win32" ? "npm.cmd" : "npm";

function usage(exitCode = 0) {
  const lines = [
    "Usage:",
    "  node scripts/prepare-release.mjs <version> [--push]",
    "",
    "Examples:",
    "  node scripts/prepare-release.mjs 0.1.11",
    "  node scripts/prepare-release.mjs 0.1.11 --push",
    "",
    "What it does:",
    "  1. Verifies the git worktree is clean",
    "  2. Syncs all repo versions to <version>",
    "  3. Runs release checks",
    "  4. Creates a release commit",
    "  5. Creates git tag v<version>",
    "  6. Optionally pushes commit + tag with --push",
  ];
  const stream = exitCode === 0 ? process.stdout : process.stderr;
  stream.write(`${lines.join("\n")}\n`);
  process.exit(exitCode);
}

function run(command, args, options = {}) {
  const result = spawnSync(command, args, {
    cwd: root,
    stdio: "inherit",
    ...options,
  });

  if (result.status !== 0) {
    const rendered = [command, ...args].join(" ");
    throw new Error(`Command failed: ${rendered}`);
  }
}

function capture(command, args) {
  const result = spawnSync(command, args, {
    cwd: root,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  });

  if (result.status !== 0) {
    const stderr = result.stderr?.trim();
    const rendered = [command, ...args].join(" ");
    throw new Error(stderr ? `${rendered}: ${stderr}` : `Command failed: ${rendered}`);
  }

  return result.stdout.trim();
}

function ensureNode18() {
  const major = Number(process.versions.node.split(".")[0]);
  if (major < 18) {
    throw new Error(`Node 18+ is required for release builds. Current Node: ${process.versions.node}`);
  }
}

function normalizeVersion(raw) {
  const version = raw.startsWith("v") ? raw.slice(1) : raw;
  if (!/^\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?$/.test(version)) {
    throw new Error(`Invalid version "${raw}". Expected semver like 0.1.11`);
  }
  return version;
}

function ensureCleanWorktree() {
  const status = capture("git", ["status", "--porcelain"]);
  if (status.length > 0) {
    throw new Error("Git worktree is not clean. Commit or stash your current changes before preparing a release.");
  }
}

function ensureTagDoesNotExist(tag) {
  const existing = capture("git", ["tag", "-l", tag]);
  if (existing === tag) {
    throw new Error(`Git tag ${tag} already exists.`);
  }
}

function ensureVersionChangesAreStaged() {
  const status = spawnSync("git", ["diff", "--cached", "--quiet", "--exit-code"], {
    cwd: root,
    stdio: "ignore",
  });

  if (status.status === 0) {
    throw new Error("No staged changes found after syncing versions.");
  }
}

const args = process.argv.slice(2);

if (args.includes("--help") || args.includes("-h")) {
  usage(0);
}

const versionArg = args.find((arg) => !arg.startsWith("--"));
if (!versionArg) {
  usage(1);
}

const push = args.includes("--push");
const version = normalizeVersion(versionArg);
const tag = `v${version}`;

try {
  ensureNode18();
  ensureCleanWorktree();
  ensureTagDoesNotExist(tag);

  run("node", ["scripts/sync-version.mjs", version]);
  run("cargo", ["test", "-p", "tama-core"]);
  run("cargo", ["check", "-p", "tama-tauri"]);
  run(npmCmd, ["--prefix", "tama-tauri/ui", "run", "build"]);

  run("git", ["add", "-u"]);
  ensureVersionChangesAreStaged();

  run("git", ["commit", "-m", `Release ${version}`]);
  run("git", ["tag", tag]);

  if (push) {
    run("git", ["push", "--follow-tags"]);
    console.log(`[prepare-release] released ${version} and pushed commit + tag`);
  } else {
    console.log(`[prepare-release] prepared ${version}`);
    console.log(`[prepare-release] next step: git push --follow-tags`);
  }
} catch (error) {
  const message = error instanceof Error ? error.message : String(error);
  console.error(`[prepare-release] ${message}`);
  process.exit(1);
}
