import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { z } from "zod";
import { bridge, type BridgeResponse } from "./bridge.js";

/** Format a successful bridge response as an MCP tool result. */
function successResult(response: BridgeResponse) {
  return {
    content: [{ type: "text" as const, text: JSON.stringify(response.state) }],
  };
}

/** Format a failed bridge response as an MCP tool error result. */
function errorResult(message: string) {
  return {
    content: [{ type: "text" as const, text: JSON.stringify({ error: message }) }],
    isError: true,
  };
}

/** Send a bridge request and return a formatted MCP tool result. */
async function callBridge(action: string, params?: Record<string, unknown>) {
  const response = await bridge.sendRequest(action, params);
  if (response.ok) {
    return successResult(response);
  }
  return errorResult(response.error ?? "Unknown error");
}

/**
 * Register all tama96 MCP tools on the given server.
 *
 * Tools: feed, play_game, discipline, give_medicine, clean_poop,
 *        toggle_lights, get_status
 */
export function registerTools(server: McpServer) {
  server.tool(
    "feed",
    "Feed the pet a meal or snack",
    { type: z.enum(["meal", "snack"]) },
    async ({ type }) => {
      const action = type === "meal" ? "feed_meal" : "feed_snack";
      return callBridge(action);
    },
  );

  server.tool(
    "play_game",
    "Play the left/right guessing game with 5 moves",
    { moves: z.array(z.enum(["Left", "Right"])).length(5) },
    async ({ moves }) => {
      return callBridge("play_game", { moves });
    },
  );

  server.tool("discipline", "Discipline the pet (only works when a discipline call is pending)", async () => {
    return callBridge("discipline");
  });

  server.tool("give_medicine", "Give the pet medicine (two doses cure sickness)", async () => {
    return callBridge("give_medicine");
  });

  server.tool("clean_poop", "Clean one poop from the pet's area", async () => {
    return callBridge("clean_poop");
  });

  server.tool("toggle_lights", "Toggle the lights on or off", async () => {
    return callBridge("toggle_lights");
  });

  server.tool("get_status", "Get the current pet state", async () => {
    return callBridge("get_status");
  });
}
