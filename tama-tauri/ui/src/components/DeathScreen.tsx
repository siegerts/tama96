import type { PetState } from "../types";

interface DeathScreenProps {
  state: PetState;
  onHatchNewEgg: () => void;
}

export default function DeathScreen({ state, onHatchNewEgg }: DeathScreenProps) {
  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        justifyContent: "center",
        gap: 16,
        padding: 32,
        fontFamily: "monospace",
        minHeight: "60vh",
      }}
    >
      <span style={{ fontSize: 64, lineHeight: 1 }}>💀</span>
      <h2 style={{ margin: 0, fontSize: 18, color: "#eee" }}>Your pet has died</h2>
      <div style={{ display: "flex", flexDirection: "column", alignItems: "center", gap: 4, color: "#aaa", fontSize: 14 }}>
        <span>Character: {state.character}</span>
        <span>Final Age: {state.age} year{state.age !== 1 ? "s" : ""}</span>
      </div>
      <button
        onClick={onHatchNewEgg}
        style={{
          marginTop: 16,
          padding: "10px 24px",
          fontSize: 14,
          fontFamily: "monospace",
          background: "#4caf50",
          color: "#fff",
          border: "none",
          borderRadius: 6,
          cursor: "pointer",
        }}
        aria-label="Hatch new egg"
      >
        🥚 Hatch New Egg
      </button>
    </div>
  );
}
