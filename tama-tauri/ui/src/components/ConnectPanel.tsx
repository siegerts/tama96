import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./ConnectPanel.css";

type Client = "Codex" | "Other clients";

export default function ConnectPanel({ onClose }: { onClose: () => void }) {
  const dialog = useRef<HTMLDialogElement>(null);
  const snippet = useRef<HTMLTextAreaElement>(null);
  const [client, setClient] = useState<Client>("Codex");
  const [config, setConfig] = useState("");
  const [loadError, setLoadError] = useState("");
  const [busy, setBusy] = useState(false);
  const [registered, setRegistered] = useState(false);
  const [message, setMessage] = useState("");
  const [copyMessage, setCopyMessage] = useState("");

  useEffect(() => {
    const element = dialog.current!;
    element.showModal();
    let active = true;
    invoke<string>("get_mcp_config").then(value => {
      if (active) setConfig(value);
    }).catch(() => { if (active) setLoadError("The app could not read its MCP configuration."); });
    return () => { active = false; element.close(); };
  }, []);

  async function register() {
    setBusy(true);
    setMessage("");
    try {
      setMessage(await invoke<string>("register_with_codex"));
      setRegistered(true);
    } catch (error) {
      setMessage(String(error));
    } finally { setBusy(false); }
  }

  let text = config;
  if (client === "Codex" && config) {
    try {
      const { command, args } = JSON.parse(config).mcpServers.tama96;
      text = `[mcp_servers.tama96]\ncommand = ${JSON.stringify(command)}\nargs = ${JSON.stringify(args)}`;
    } catch { text = ""; }
  }

  async function copy() {
    try {
      await navigator.clipboard.writeText(text);
      setCopyMessage("Copied");
    } catch {
      snippet.current?.focus();
      snippet.current?.select();
      setCopyMessage("Selected. Use your usual copy shortcut.");
    }
  }

  return (
    <dialog className="connect-dialog" ref={dialog} aria-labelledby="connect-heading" onCancel={onClose}>
      <header><h2 id="connect-heading">Connect an AI tool</h2><button className="connect-close" aria-label="Close setup" onClick={onClose}>×</button></header>
      <div className="client-switch" aria-label="AI client" role="group">
        {(["Codex", "Other clients"] as const).map(item =>
          <button key={item} aria-pressed={client === item} onClick={() => { setClient(item); setCopyMessage(""); }}>{item}</button>)}
      </div>
      {client === "Codex" && <>
        <p>Add this pet to your Codex MCP settings. Keep tama96 open while Codex cares for it.</p>
        <button className="connect-primary" onClick={register} disabled={busy || registered}>{busy ? "Adding…" : registered ? "Configured" : "Add to Codex"}</button>
        {message && <p className="connect-result" role="status">{message}</p>}
        {registered && <p>Open a new Codex chat and ask “How is my tama96 pet doing?”</p>}
      </>}
      {client === "Other clients" && <p>Add the tama96 server entry to your client’s MCP configuration. Keep the desktop app open.</p>}
      <details key={client} open={client !== "Codex" || undefined}>
        <summary>{client === "Codex" ? "Manual setup" : "Server configuration"}</summary>
        <p>{client === "Codex" ? "Add this section to your Codex config.toml. Node.js 18 or newer must be on the client’s PATH." : "Merge this entry with any servers you already use."}</p>
        {text ? <>
          <textarea ref={snippet} readOnly value={text} rows={4} aria-label="MCP configuration" />
          <button className="connect-copy" onClick={copy}>Copy {client === "Codex" ? "TOML" : "JSON"}</button>
          <small role="status">{copyMessage}</small>
        </> : <p role="status">{loadError || (config ? "The server configuration could not be read." : "Loading configuration…")}</p>}
      </details>
    </dialog>
  );
}
