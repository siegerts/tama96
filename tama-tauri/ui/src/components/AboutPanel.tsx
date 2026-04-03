import { useEffect, useState } from "react";
import { getVersion } from "@tauri-apps/api/app";
import { open } from "@tauri-apps/plugin-shell";

interface AboutPanelProps {
  onClose: () => void;
}

function ExtLink({ href, children }: { href: string; children: React.ReactNode }) {
  return (
    <a
      href="#"
      onClick={(e) => { e.preventDefault(); open(href); }}
      style={linkStyle}
    >
      {children}
    </a>
  );
}

export default function AboutPanel({ onClose }: AboutPanelProps) {
  const [version, setVersion] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;

    getVersion()
      .then((value) => {
        if (!cancelled) setVersion(value);
      })
      .catch(() => {
        if (!cancelled) setVersion("unknown");
      });

    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <div style={overlayStyle} onClick={onClose}>
      <div style={panelStyle} onClick={(e) => e.stopPropagation()}>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 12 }}>
          <span style={{ fontSize: 16, fontFamily: "monospace", color: "#eee" }}>tama96</span>
          <button onClick={onClose} style={closeBtnStyle} aria-label="Close">✕</button>
        </div>

        <div style={{ fontSize: 11, fontFamily: "monospace", color: "#999", lineHeight: 2 }}>
          <div>{version ? `v${version}` : "version..."}</div>
          <div>Made by <ExtLink href="https://x.com/siegerts">@siegerts</ExtLink></div>
          <div>Built with <ExtLink href="https://kiro.dev/">Kiro</ExtLink></div>
          <div><ExtLink href="https://github.com/siegerts/tama96">GitHub</ExtLink></div>
        </div>

        <div style={{ marginTop: 12, paddingTop: 10, borderTop: "1px solid #333", fontSize: 10, fontFamily: "monospace", color: "#666", lineHeight: 1.6 }}>
          <div>"Tamagotchi" is a registered trademark of Bandai Co., Ltd.</div>
          <div>Not affiliated with or endorsed by Bandai.</div>
          <div style={{ marginTop: 6 }}>MIT License</div>
        </div>
      </div>
    </div>
  );
}

const overlayStyle: React.CSSProperties = {
  position: "fixed",
  inset: 0,
  background: "rgba(0,0,0,0.5)",
  display: "flex",
  justifyContent: "center",
  alignItems: "center",
  zIndex: 100,
};

const panelStyle: React.CSSProperties = {
  background: "#1e1e1e",
  border: "1px solid #444",
  borderRadius: 8,
  padding: 16,
  width: 220,
  color: "#eee",
};

const closeBtnStyle: React.CSSProperties = {
  background: "none",
  border: "none",
  color: "#aaa",
  fontSize: 16,
  cursor: "pointer",
  padding: "2px 6px",
};

const linkStyle: React.CSSProperties = {
  color: "#7bafd4",
  textDecoration: "none",
};
