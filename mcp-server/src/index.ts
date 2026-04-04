import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import { registerTools } from "./tools.js";
import { registerResources } from "./resources.js";
import { bridge } from "./bridge.js";

const currentDir = path.dirname(fileURLToPath(import.meta.url));

function readVersion(): string {
  // Desktop bundles flatten the MCP server files into one resource directory,
  // while local development keeps dist/ under the package root.
  const candidates = [
    path.join(currentDir, "../package.json"),
    path.join(currentDir, "package.json"),
  ];

  for (const candidate of candidates) {
    try {
      const pkg = JSON.parse(fs.readFileSync(candidate, "utf8")) as { version?: string };
      if (pkg.version) {
        return pkg.version;
      }
    } catch {
      // Try the next layout.
    }
  }

  return "0.0.0";
}

const version = readVersion();

const server = new McpServer({
  name: "tama96",
  version,
});

registerTools(server);
registerResources(server);

async function main() {
  try {
    await bridge.connect();
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    console.error("[tama96] Bridge connection failed: " + message);
    console.error("[tama96] Server starting anyway. Tools retry on use.");
  }

  const transport = new StdioServerTransport();
  await server.connect(transport);
}

main().catch(console.error);
