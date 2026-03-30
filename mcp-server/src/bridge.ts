import * as net from "node:net";
import * as fs from "node:fs";
import * as path from "node:path";
import * as os from "node:os";

/** JSON request sent to the Tauri TCP socket server. */
export interface BridgeRequest {
  action: string;
  params?: Record<string, unknown>;
}

/** JSON response received from the Tauri TCP socket server. */
export interface BridgeResponse {
  ok: boolean;
  state?: Record<string, unknown>;
  error?: string;
}

const PORT_FILE = path.join(os.homedir(), ".tama96", "mcp_port");
const HOST = "127.0.0.1";
const MAX_RETRIES = 3;
const RETRY_DELAY_MS = 1000;

/**
 * TCP bridge client that communicates with the Tauri backend
 * via newline-delimited JSON over a localhost TCP socket.
 */
export class Bridge {
  private socket: net.Socket | null = null;
  private buffer = "";
  private pending: {
    resolve: (value: BridgeResponse) => void;
    reject: (reason: Error) => void;
  } | null = null;

  /** Read the port number from ~/.tama96/mcp_port */
  private readPort(): number {
    const raw = fs.readFileSync(PORT_FILE, "utf-8").trim();
    const port = parseInt(raw, 10);
    if (isNaN(port) || port <= 0 || port > 65535) {
      throw new Error(`Invalid port in ${PORT_FILE}: "${raw}"`);
    }
    return port;
  }

  /** Connect to the Tauri TCP socket server. */
  async connect(): Promise<void> {
    if (this.socket && !this.socket.destroyed) {
      return; // already connected
    }

    const port = this.readPort();

    return new Promise<void>((resolve, reject) => {
      let connected = false;

      const socket = net.createConnection({ host: HOST, port }, () => {
        connected = true;
        this.socket = socket;
        this.buffer = "";
        resolve();
      });

      socket.setEncoding("utf-8");

      socket.on("data", (chunk: string) => {
        this.buffer += chunk;
        this.processBuffer();
      });

      socket.on("error", (err) => {
        if (!connected) {
          // Connection failed — reject the connect() promise
          reject(err);
          return;
        }
        // Runtime error on an established connection
        if (this.pending) {
          this.pending.reject(err);
          this.pending = null;
        }
        this.socket = null;
      });

      socket.on("close", () => {
        if (this.pending) {
          this.pending.reject(new Error("Connection closed"));
          this.pending = null;
        }
        this.socket = null;
      });
    });
  }

  /** Process buffered data, splitting on newlines to extract complete JSON responses. */
  private processBuffer(): void {
    let newlineIdx: number;
    while ((newlineIdx = this.buffer.indexOf("\n")) !== -1) {
      const line = this.buffer.slice(0, newlineIdx).trim();
      this.buffer = this.buffer.slice(newlineIdx + 1);

      if (line.length === 0) continue;

      if (this.pending) {
        try {
          const response = JSON.parse(line) as BridgeResponse;
          this.pending.resolve(response);
        } catch (err) {
          this.pending.reject(
            new Error(`Invalid JSON from server: ${line}`)
          );
        }
        this.pending = null;
      }
    }
  }

  /** Disconnect from the TCP socket. */
  disconnect(): void {
    if (this.socket) {
      this.socket.destroy();
      this.socket = null;
    }
    this.buffer = "";
    if (this.pending) {
      this.pending.reject(new Error("Bridge disconnected"));
      this.pending = null;
    }
  }

  /**
   * Send a JSON request to the Tauri backend and wait for the response.
   * Automatically reconnects with retry logic on connection failure.
   */
  async sendRequest(
    action: string,
    params?: Record<string, unknown>
  ): Promise<BridgeResponse> {
    let lastError: Error | null = null;

    for (let attempt = 0; attempt <= MAX_RETRIES; attempt++) {
      try {
        // Ensure we're connected (reconnect if needed)
        await this.connect();

        const request: BridgeRequest = { action };
        if (params !== undefined) {
          request.params = params;
        }

        const response = await this.sendRaw(request);
        return response;
      } catch (err) {
        lastError = err instanceof Error ? err : new Error(String(err));
        // Clean up broken connection before retry
        this.disconnect();

        if (attempt < MAX_RETRIES) {
          await this.delay(RETRY_DELAY_MS * (attempt + 1));
        }
      }
    }

    throw new Error(
      `Bridge request failed after ${MAX_RETRIES + 1} attempts: ${lastError?.message}`
    );
  }

  /** Send a raw JSON request and wait for the newline-delimited response. */
  private sendRaw(request: BridgeRequest): Promise<BridgeResponse> {
    return new Promise<BridgeResponse>((resolve, reject) => {
      if (!this.socket || this.socket.destroyed) {
        reject(new Error("Not connected"));
        return;
      }

      if (this.pending) {
        reject(new Error("Another request is already in flight"));
        return;
      }

      this.pending = { resolve, reject };

      const payload = JSON.stringify(request) + "\n";
      this.socket.write(payload, (err) => {
        if (err) {
          this.pending = null;
          reject(err);
        }
      });
    });
  }

  private delay(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }
}

/** Singleton bridge instance for use across the MCP server. */
export const bridge = new Bridge();
