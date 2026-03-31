import { useCallback, useEffect, useState } from "react";
import { usePetState } from "./hooks/usePetState";
import PetDisplay from "./components/PetDisplay";
import ActionBar from "./components/ActionBar";
import DeathScreen from "./components/DeathScreen";
import PermissionsPanel from "./components/PermissionsPanel";

const SHELL_COLORS = [
  { name: "Teal", value: "#5b9a9a" },
  { name: "Rose", value: "#c48b9f" },
  { name: "Lavender", value: "#9b8ec4" },
  { name: "Mint", value: "#7bc4a8" },
  { name: "Peach", value: "#d4a574" },
  { name: "Sky", value: "#7bafd4" },
  { name: "Coral", value: "#d4837b" },
  { name: "Slate", value: "#7a8a8a" },
  { name: "Lilac", value: "#b89ad4" },
  { name: "Butter", value: "#d4c97b" },
];

const STORAGE_KEY = "tama96_shell_color";

function loadShellColor(): string {
  try {
    return localStorage.getItem(STORAGE_KEY) ?? SHELL_COLORS[0].value;
  } catch {
    return SHELL_COLORS[0].value;
  }
}

function App() {
  const {
    state, loading, error,
    feedMeal, feedSnack, playGame, discipline,
    giveMedicine, cleanPoop, toggleLights, hatchNewEgg,
  } = usePetState();
  const [showSettings, setShowSettings] = useState(false);
  const [showColors, setShowColors] = useState(false);
  const [shellColor, setShellColor] = useState(loadShellColor);

  const pickColor = useCallback((color: string) => {
    setShellColor(color);
    try { localStorage.setItem(STORAGE_KEY, color); } catch { /* noop */ }
    setShowColors(false);
  }, []);

  useEffect(() => {
    if (!showColors) return;
    const handler = () => setShowColors(false);
    const timer = setTimeout(() => document.addEventListener("click", handler), 0);
    return () => { clearTimeout(timer); document.removeEventListener("click", handler); };
  }, [showColors]);

  if (loading) {
    return (
      <div style={{ ...shellStyle, background: shellColor }}>
        <p style={{ fontFamily: "monospace", color: "#fff8", fontSize: 11, margin: "auto" }}>loading...</p>
      </div>
    );
  }

  if (error) {
    return (
      <div style={{ ...shellStyle, background: shellColor }}>
        <p style={{ fontFamily: "monospace", color: "#d44", fontSize: 11, margin: "auto" }}>{error}</p>
      </div>
    );
  }

  if (!state) return null;

  return (
    <div style={{ ...shellStyle, background: shellColor }}>
      {!state.is_alive ? (
        <DeathScreen state={state} onHatchNewEgg={hatchNewEgg} />
      ) : (
        <PetDisplay state={state} />
      )}

      {state.is_alive && (
        <ActionBar
          state={state}
          feedMeal={feedMeal}
          feedSnack={feedSnack}
          playGame={playGame}
          discipline={discipline}
          giveMedicine={giveMedicine}
          cleanPoop={cleanPoop}
          toggleLights={toggleLights}
        />
      )}

      <div style={bottomRow}>
        <div style={{ position: "relative" }}>
          <button
            onClick={(e) => { e.stopPropagation(); setShowColors(v => !v); }}
            style={dotBtn(shellColor)}
            aria-label="Change shell color"
            title="Shell color"
          />
          {showColors && (
            <div style={colorPicker} onClick={(e) => e.stopPropagation()}>
              {SHELL_COLORS.map((c) => (
                <button
                  key={c.value}
                  onClick={() => pickColor(c.value)}
                  style={colorSwatch(c.value, c.value === shellColor)}
                  aria-label={c.name}
                  title={c.name}
                />
              ))}
            </div>
          )}
        </div>
        <button
          onClick={() => setShowSettings(true)}
          style={cfgStyle}
          aria-label="Agent permissions"
          title="Agent Permissions"
        >
          cfg
        </button>
      </div>

      {showSettings && <PermissionsPanel onClose={() => setShowSettings(false)} />}
    </div>
  );
}

const shellStyle: React.CSSProperties = {
  width: "100%",
  height: "100vh",
  display: "flex",
  flexDirection: "column",
  alignItems: "center",
  padding: "6px",
  overflow: "hidden",
};

const bottomRow: React.CSSProperties = {
  display: "flex",
  alignItems: "center",
  gap: 8,
  marginTop: "auto",
  marginBottom: 2,
};

const cfgStyle: React.CSSProperties = {
  background: "none",
  border: "none",
  cursor: "pointer",
  fontSize: 9,
  fontFamily: "monospace",
  color: "#0004",
  padding: "2px 4px",
};

const dotBtn = (color: string): React.CSSProperties => ({
  width: 14,
  height: 14,
  borderRadius: "50%",
  border: "2px solid #0003",
  background: color,
  cursor: "pointer",
  padding: 0,
  filter: "brightness(0.8)",
});

const colorPicker: React.CSSProperties = {
  position: "absolute",
  bottom: "100%",
  left: "50%",
  transform: "translateX(-50%)",
  marginBottom: 6,
  display: "flex",
  flexWrap: "wrap",
  gap: 4,
  padding: 6,
  background: "#222e",
  borderRadius: 8,
  width: 110,
  justifyContent: "center",
};

const colorSwatch = (color: string, active: boolean): React.CSSProperties => ({
  width: 18,
  height: 18,
  borderRadius: "50%",
  border: active ? "2px solid #fff" : "2px solid #0003",
  background: color,
  cursor: "pointer",
  padding: 0,
});

export default App;
