import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import * as fs from "node:fs";
import * as path from "node:path";
import * as os from "node:os";
import { bridge } from "./bridge.js";

const PERMISSIONS_FILE = path.join(os.homedir(), ".tama96", "permissions.json");

/** Static text description of the P1 evolution branching matrix. */
const EVOLUTION_CHART = `# Tamagotchi P1 Evolution Chart

## Life Stages
Egg → Baby → Child → Teen → Adult → Special (rare) / Dead

## Stage Transitions
- Egg → Baby (Babytchi): after 5 minutes
- Baby → Child (Marutchi): after 65 minutes
- Child → Teen: at age 3 (3 real days)
- Teen → Adult: at age 6 (6 real days)
- Adult → Special: Maskutchi (Tamatchi T2 path) after 4 additional days → Oyajitchi

## Child → Teen Branching
- 0–2 care mistakes → Tamatchi
- 3+ care mistakes  → Kuchitamatchi
- 0–2 discipline mistakes → Type1
- 3+ discipline mistakes  → Type2

## Teen → Adult Branching Matrix

### Tamatchi Type1
| Care Mistakes | Discipline Mistakes | Adult       |
|---------------|---------------------|-------------|
| 0–2           | 0                   | Mametchi    |
| 0–2           | 1                   | Ginjirotchi |
| 0–2           | 2+                  | Maskutchi   |
| 3+            | 0–1                 | Kuchipatchi |
| 3+            | 2–3                 | Nyorotchi   |
| 3+            | 4+                  | Tarakotchi  |

### Tamatchi Type2
| Care Mistakes | Discipline Mistakes | Adult       |
|---------------|---------------------|-------------|
| 0–3           | 2+                  | Maskutchi   |
| 3+            | 0–1                 | Kuchipatchi |
| 3+            | 2–3                 | Nyorotchi   |
| 3+            | 4+                  | Tarakotchi  |
| other         | other               | Nyorotchi   |

### Kuchitamatchi (Type1 & Type2)
| Discipline Mistakes | Adult       |
|---------------------|-------------|
| 0–1                 | Kuchipatchi |
| 2–3                 | Nyorotchi   |
| 4+                  | Tarakotchi  |

## Special Evolution
- Maskutchi (from Tamatchi Type2 path) → Oyajitchi after 4 days in Adult stage
`;

/**
 * Register all tama96 MCP resources on the given server.
 *
 * Resources: pet://status, pet://evolution-chart, pet://permissions
 */
export function registerResources(server: McpServer) {
  // pet://status — current pet state summary via bridge
  server.resource(
    "pet-status",
    "pet://status",
    { description: "Current pet state summary", mimeType: "application/json" },
    async (uri) => {
      const response = await bridge.sendRequest("get_status");
      const text = response.ok
        ? JSON.stringify(response.state, null, 2)
        : JSON.stringify({ error: response.error ?? "Failed to get status" });

      return {
        contents: [{ uri: uri.href, text, mimeType: "application/json" }],
      };
    },
  );

  // pet://evolution-chart — static P1 evolution branching matrix
  server.resource(
    "evolution-chart",
    "pet://evolution-chart",
    { description: "P1 evolution branching matrix", mimeType: "text/markdown" },
    async (uri) => {
      return {
        contents: [{ uri: uri.href, text: EVOLUTION_CHART, mimeType: "text/markdown" }],
      };
    },
  );

  // pet://permissions — current agent permission configuration
  server.resource(
    "pet-permissions",
    "pet://permissions",
    { description: "Current agent permission configuration", mimeType: "application/json" },
    async (uri) => {
      let text: string;
      try {
        text = fs.readFileSync(PERMISSIONS_FILE, "utf-8");
        // Validate it's valid JSON by parsing, then pretty-print
        const parsed = JSON.parse(text);
        text = JSON.stringify(parsed, null, 2);
      } catch {
        text = JSON.stringify({ error: "Permissions file not found or unreadable" });
      }

      return {
        contents: [{ uri: uri.href, text, mimeType: "application/json" }],
      };
    },
  );
}
