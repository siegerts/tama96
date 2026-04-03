import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(__dirname, "..");

const tauriConfigPath = path.join(root, "tama-tauri", "src-tauri", "tauri.conf.json");
const cargoTomlPaths = [
  path.join(root, "tama-core", "Cargo.toml"),
  path.join(root, "tama-tui", "Cargo.toml"),
  path.join(root, "tama-tauri", "src-tauri", "Cargo.toml"),
];
const packageJsonPaths = [
  path.join(root, "mcp-server", "package.json"),
  path.join(root, "tama-tauri", "ui", "package.json"),
  path.join(root, "website", "package.json"),
];
const packageLockPaths = [
  path.join(root, "mcp-server", "package-lock.json"),
  path.join(root, "tama-tauri", "ui", "package-lock.json"),
  path.join(root, "website", "package-lock.json"),
];

function validateVersion(version) {
  if (!/^\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?$/.test(version)) {
    throw new Error(`Invalid version "${version}". Expected semver like 0.1.10 or 0.1.10-beta.1`);
  }
}

function readText(filePath) {
  return fs.readFileSync(filePath, "utf8");
}

function writeIfChanged(filePath, nextText) {
  const prevText = readText(filePath);
  if (prevText !== nextText) {
    fs.writeFileSync(filePath, nextText);
  }
}

function readJson(filePath) {
  return JSON.parse(readText(filePath));
}

function writeJson(filePath, data) {
  writeIfChanged(filePath, `${JSON.stringify(data, null, 2)}\n`);
}

function detectVersion() {
  const cliValue = process.argv[2]?.trim();
  if (cliValue) {
    return cliValue.startsWith("v") ? cliValue.slice(1) : cliValue;
  }

  const tauriConfig = readJson(tauriConfigPath);
  if (typeof tauriConfig.version !== "string") {
    throw new Error(`Could not detect version from ${tauriConfigPath}`);
  }
  return tauriConfig.version;
}

function updateCargoToml(filePath, version) {
  const source = readText(filePath);
  let replaced = false;
  const next = source.replace(/^version = ".*"$/m, () => {
    replaced = true;
    return `version = "${version}"`;
  });

  if (!replaced) {
    throw new Error(`Could not find package version in ${filePath}`);
  }

  writeIfChanged(filePath, next);
}

function updatePackageJson(filePath, version) {
  const data = readJson(filePath);
  data.version = version;
  writeJson(filePath, data);
}

function updatePackageLock(filePath, version) {
  const data = readJson(filePath);
  data.version = version;
  if (data.packages && data.packages[""]) {
    data.packages[""].version = version;
  }
  writeJson(filePath, data);
}

function updateTauriConfig(version) {
  const data = readJson(tauriConfigPath);
  data.version = version;
  writeJson(tauriConfigPath, data);
}

const version = detectVersion();
validateVersion(version);

for (const filePath of cargoTomlPaths) {
  updateCargoToml(filePath, version);
}

for (const filePath of packageJsonPaths) {
  updatePackageJson(filePath, version);
}

for (const filePath of packageLockPaths) {
  updatePackageLock(filePath, version);
}

updateTauriConfig(version);

console.log(`[sync-version] synced repo version to ${version}`);
