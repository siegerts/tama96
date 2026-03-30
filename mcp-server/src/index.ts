import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import { registerTools } from "./tools.js";
import { registerResources } from "./resources.js";
import { bridge } from "./bridge.js";

const server = new McpServer({
  name: "tama96",
  version: "0.1.0",
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
