import type { PetState } from "../types";

interface DeathScreenProps {
  state: PetState;
  onHatchNewEgg: () => void;
}

const GRAVE_ART = [
  "     .  .     ",
  "    . -- .    ",
  "   /  R   \\   ",
  "  |  I  P  |  ",
  "  |________|  ",
  "  ^^^^^^^^^^  ",
];

export default function DeathScreen({ state, onHatchNewEgg }: DeathScreenProps) {
  return (
    <div style={containerStyle}>
      <pre style={artStyle}>{GRAVE_ART.join("\n")}</pre>
      <div style={textStyle}>YOUR PET HAS DIED</div>
      <div style={infoStyle}>
        <span>{state.character}</span>
        <span>FINAL AGE: {state.age}</span>
      </div>
      <button onClick={onHatchNewEgg} style={buttonStyle} aria-label="Hatch new egg">
        [ HATCH NEW EGG ]
      </button>
    </div>
  );
}

const containerStyle: React.CSSProperties = {
  display: "flex",
  flexDirection: "column",
  alignItems: "center",
  justifyContent: "center",
  gap: 16,
  padding: 32,
  fontFamily: "monospace",
  minHeight: "60vh",
};

const artStyle: React.CSSProperties = {
  margin: 0,
  fontSize: 14,
  lineHeight: 1.3,
  color: "#666",
  textAlign: "center",
};

const textStyle: React.CSSProperties = {
  fontSize: 14,
  letterSpacing: 2,
  color: "#888",
};

const infoStyle: React.CSSProperties = {
  display: "flex",
  flexDirection: "column",
  alignItems: "center",
  gap: 4,
  color: "#666",
  fontSize: 12,
  letterSpacing: 1,
};

const buttonStyle: React.CSSProperties = {
  marginTop: 16,
  padding: "8px 20px",
  fontSize: 12,
  fontFamily: "monospace",
  letterSpacing: 2,
  background: "#2a3a2a",
  color: "#8a8",
  border: "1px solid #555",
  borderRadius: 3,
  cursor: "pointer",
};
