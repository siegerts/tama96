import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { ActionType, AgentPermissions, ActionPermission } from "../types";

const ACTION_TYPES: ActionType[] = [
  "FeedMeal",
  "FeedSnack",
  "PlayGame",
  "Discipline",
  "GiveMedicine",
  "CleanPoop",
  "ToggleLights",
  "GetStatus",
];

const ACTION_LABELS: Record<ActionType, string> = {
  FeedMeal: "Feed Meal",
  FeedSnack: "Feed Snack",
  PlayGame: "Play Game",
  Discipline: "Discipline",
  GiveMedicine: "Give Medicine",
  CleanPoop: "Clean Poop",
  ToggleLights: "Toggle Lights",
  GetStatus: "Get Status",
};

interface PermissionsPanelProps {
  onClose: () => void;
}

export default function PermissionsPanel({ onClose }: PermissionsPanelProps) {
  const [permissions, setPermissions] = useState<AgentPermissions | null>(null);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(async () => {
    try {
      const p = await invoke<AgentPermissions>("get_permissions");
      setPermissions(p);
      setError(null);
    } catch (e) {
      setError(String(e));
    }
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  const save = useCallback(async (updated: AgentPermissions) => {
    setSaving(true);
    try {
      const saved = await invoke<AgentPermissions>("update_permissions", {
        newPermissions: updated,
      });
      setPermissions(saved);
      setError(null);
    } catch (e) {
      setError(String(e));
    } finally {
      setSaving(false);
    }
  }, []);

  const toggleMaster = () => {
    if (!permissions) return;
    const updated = { ...permissions, enabled: !permissions.enabled };
    setPermissions(updated);
    save(updated);
  };

  const toggleAction = (action: ActionType) => {
    if (!permissions) return;
    const current: ActionPermission = permissions.allowed_actions[action] ?? {
      allowed: true,
      max_per_hour: null,
    };
    const updated: AgentPermissions = {
      ...permissions,
      allowed_actions: {
        ...permissions.allowed_actions,
        [action]: { ...current, allowed: !current.allowed },
      },
    };
    setPermissions(updated);
    save(updated);
  };

  const setRateLimit = (action: ActionType, value: string) => {
    if (!permissions) return;
    const current: ActionPermission = permissions.allowed_actions[action] ?? {
      allowed: true,
      max_per_hour: null,
    };
    const num = value === "" ? null : Math.max(1, parseInt(value, 10) || 1);
    const updated: AgentPermissions = {
      ...permissions,
      allowed_actions: {
        ...permissions.allowed_actions,
        [action]: { ...current, max_per_hour: num },
      },
    };
    setPermissions(updated);
    save(updated);
  };

  if (!permissions) {
    return (
      <div style={overlayStyle}>
        <div style={panelStyle}>
          <p>{error ? `Error: ${error}` : "Loading permissions…"}</p>
        </div>
      </div>
    );
  }

  return (
    <div style={overlayStyle}>
      <div style={panelStyle}>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 8 }}>
          <h2 style={{ margin: 0, fontSize: 16, fontFamily: "monospace" }}>Settings</h2>
          <button onClick={onClose} style={closeBtnStyle} aria-label="Close settings">✕</button>
        </div>

        {/* MCP Config section */}
        <div style={{ marginBottom: 16, paddingBottom: 12, borderBottom: "1px solid #444" }}>
          <h3 style={{ margin: "0 0 6px", fontSize: 13, fontFamily: "monospace" }}>Connect an AI tool</h3>
          <p style={{ fontSize: 11, color: "#999", fontFamily: "monospace", margin: "0 0 8px", lineHeight: 1.5 }}>
            Copy the config below and paste it into your AI tool's MCP settings
            (Kiro, Cursor, etc). The app needs to be running for the
            connection to work.
          </p>
          <CopyMcpConfig />
        </div>

        {/* Agent Permissions */}
        <h3 style={{ margin: "0 0 6px", fontSize: 13, fontFamily: "monospace" }}>Agent Permissions</h3>
        <p style={{ fontSize: 11, color: "#999", fontFamily: "monospace", margin: "0 0 12px", lineHeight: 1.5 }}>
          Control what the MCP agent can do with your pet.
          When connected via an AI tool, the agent can perform
          actions on your behalf. Toggle individual actions
          on/off and set rate limits per hour.
        </p>

        {error && <p style={{ color: "#f44336", fontSize: 12, margin: "0 0 8px" }}>{error}</p>}

        {/* Master switch */}
        <div style={{ display: "flex", alignItems: "center", gap: 10, marginBottom: 16, padding: "8px 0", borderBottom: "1px solid #444" }}>
          <label style={{ fontSize: 13, fontFamily: "monospace", flex: 1 }}>Agent Access</label>
          <button
            onClick={toggleMaster}
            style={toggleBtnStyle(permissions.enabled)}
            aria-label="Toggle agent access"
          >
            {permissions.enabled ? "ON" : "OFF"}
          </button>
        </div>

        {/* Per-action settings */}
        <div style={{ display: "flex", flexDirection: "column", gap: 8, opacity: permissions.enabled ? 1 : 0.4 }}>
          {ACTION_TYPES.map((action) => {
            const perm: ActionPermission = permissions.allowed_actions[action] ?? {
              allowed: true,
              max_per_hour: null,
            };
            return (
              <div key={action} style={actionRowStyle}>
                <span style={{ fontSize: 12, fontFamily: "monospace", flex: 1, minWidth: 90 }}>
                  {ACTION_LABELS[action]}
                </span>
                <button
                  onClick={() => toggleAction(action)}
                  disabled={!permissions.enabled}
                  style={toggleBtnStyle(perm.allowed)}
                  aria-label={`Toggle ${ACTION_LABELS[action]}`}
                >
                  {perm.allowed ? "Allow" : "Deny"}
                </button>
                <input
                  type="number"
                  min={1}
                  placeholder="∞"
                  value={perm.max_per_hour ?? ""}
                  onChange={(e) => setRateLimit(action, e.target.value)}
                  disabled={!permissions.enabled || !perm.allowed}
                  style={rateLimitInputStyle}
                  aria-label={`Rate limit for ${ACTION_LABELS[action]}`}
                  title="Max per hour"
                />
                <span style={{ fontSize: 10, color: "#888", minWidth: 24 }}>/hr</span>
              </div>
            );
          })}
        </div>

        {saving && <p style={{ fontSize: 11, color: "#888", marginTop: 8 }}>Saving…</p>}
      </div>
    </div>
  );
}

function CopyMcpConfig() {
  const [config, setConfig] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    invoke<string>("get_mcp_config").then(setConfig).catch(() => setConfig(null));
  }, []);

  const handleCopy = async () => {
    if (!config) return;
    try {
      await navigator.clipboard.writeText(config);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      // Fallback
      const ta = document.createElement("textarea");
      ta.value = config;
      document.body.appendChild(ta);
      ta.select();
      document.execCommand("copy");
      document.body.removeChild(ta);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  if (!config) {
    return <p style={{ fontSize: 11, color: "#666", fontFamily: "monospace" }}>MCP config not available yet. Restart the app.</p>;
  }

  return (
    <div>
      <pre style={configPreStyle}>{config}</pre>
      <button onClick={handleCopy} style={copyBtnStyle}>
        {copied ? "Copied" : "Copy to clipboard"}
      </button>
    </div>
  );
}

const configPreStyle: React.CSSProperties = {
  background: "#111",
  border: "1px solid #444",
  borderRadius: 4,
  padding: 8,
  fontSize: 10,
  fontFamily: "monospace",
  color: "#aaa",
  overflowX: "auto",
  margin: "0 0 8px",
  whiteSpace: "pre-wrap",
  wordBreak: "break-all",
};

const copyBtnStyle: React.CSSProperties = {
  padding: "4px 12px",
  border: "1px solid #555",
  borderRadius: 4,
  background: "#2e7d32",
  color: "#eee",
  cursor: "pointer",
  fontSize: 11,
  fontFamily: "monospace",
};

const overlayStyle: React.CSSProperties = {
  position: "fixed",
  inset: 0,
  background: "rgba(0,0,0,0.6)",
  display: "flex",
  justifyContent: "center",
  alignItems: "center",
  zIndex: 100,
};

const panelStyle: React.CSSProperties = {
  background: "#1e1e1e",
  border: "1px solid #555",
  borderRadius: 8,
  padding: 20,
  width: 360,
  maxHeight: "80vh",
  overflowY: "auto",
  color: "#eee",
};

const closeBtnStyle: React.CSSProperties = {
  background: "none",
  border: "none",
  color: "#aaa",
  fontSize: 18,
  cursor: "pointer",
  padding: "2px 6px",
};

const toggleBtnStyle = (active: boolean): React.CSSProperties => ({
  padding: "3px 10px",
  border: "1px solid #555",
  borderRadius: 4,
  background: active ? "#2e7d32" : "#555",
  color: "#eee",
  cursor: "pointer",
  fontSize: 11,
  fontFamily: "monospace",
  minWidth: 48,
});

const actionRowStyle: React.CSSProperties = {
  display: "flex",
  alignItems: "center",
  gap: 8,
};

const rateLimitInputStyle: React.CSSProperties = {
  width: 48,
  padding: "2px 4px",
  border: "1px solid #555",
  borderRadius: 4,
  background: "#2a2a2a",
  color: "#eee",
  fontSize: 12,
  fontFamily: "monospace",
  textAlign: "center",
};
