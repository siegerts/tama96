import { useState } from "react";
import type { PetState, Choice } from "../types";

interface ActionBarProps {
  state: PetState;
  feedMeal: () => Promise<void>;
  feedSnack: () => Promise<void>;
  playGame: (moves: Choice[]) => Promise<unknown>;
  discipline: () => Promise<void>;
  giveMedicine: () => Promise<void>;
  cleanPoop: () => Promise<void>;
  toggleLights: () => Promise<void>;
}

function randomMoves(): Choice[] {
  return Array.from({ length: 5 }, () => (Math.random() < 0.5 ? "Left" : "Right"));
}

export default function ActionBar({
  state,
  feedMeal,
  feedSnack,
  playGame,
  discipline,
  giveMedicine,
  cleanPoop,
  toggleLights,
}: ActionBarProps) {
  const [feedOpen, setFeedOpen] = useState(false);

  const dead = !state.is_alive;
  const sleeping = state.is_sleeping;

  const btnStyle = (disabled: boolean): React.CSSProperties => ({
    display: "flex",
    flexDirection: "column",
    alignItems: "center",
    gap: 2,
    padding: "8px 6px",
    border: "1px solid #555",
    borderRadius: 6,
    background: disabled ? "#2a2a2a" : "#333",
    color: disabled ? "#666" : "#eee",
    cursor: disabled ? "not-allowed" : "pointer",
    fontSize: 11,
    fontFamily: "monospace",
    minWidth: 56,
    opacity: disabled ? 0.5 : 1,
    position: "relative" as const,
  });

  const iconStyle: React.CSSProperties = { fontSize: 20, lineHeight: 1 };

  const handleFeedClick = () => {
    if (dead || sleeping) return;
    setFeedOpen((o) => !o);
  };

  const handleMeal = async () => {
    setFeedOpen(false);
    await feedMeal();
  };

  const handleSnack = async () => {
    setFeedOpen(false);
    await feedSnack();
  };

  const handleGame = async () => {
    if (dead || sleeping) return;
    await playGame(randomMoves());
  };

  return (
    <div
      style={{
        display: "flex",
        flexWrap: "wrap",
        justifyContent: "center",
        gap: 6,
        padding: "12px 8px",
        maxWidth: 500,
        width: "100%",
      }}
    >
      {/* Feed (with submenu) */}
      <div style={{ position: "relative" }}>
        <button
          style={btnStyle(dead || sleeping)}
          disabled={dead || sleeping}
          onClick={handleFeedClick}
          aria-label="Feed"
          aria-expanded={feedOpen}
        >
          <span style={iconStyle}>🍽️</span>
          <span>Feed</span>
        </button>
        {feedOpen && !dead && !sleeping && (
          <div
            style={{
              position: "absolute",
              bottom: "100%",
              left: "50%",
              transform: "translateX(-50%)",
              marginBottom: 4,
              display: "flex",
              gap: 4,
              background: "#222",
              border: "1px solid #555",
              borderRadius: 6,
              padding: 4,
              zIndex: 10,
            }}
          >
            <button
              style={{ ...btnStyle(false), minWidth: 48 }}
              onClick={handleMeal}
              aria-label="Feed meal"
            >
              <span style={iconStyle}>🍚</span>
              <span>Meal</span>
            </button>
            <button
              style={{ ...btnStyle(false), minWidth: 48 }}
              onClick={handleSnack}
              aria-label="Feed snack"
            >
              <span style={iconStyle}>🍬</span>
              <span>Snack</span>
            </button>
          </div>
        )}
      </div>

      {/* Light */}
      <button
        style={btnStyle(dead)}
        disabled={dead}
        onClick={() => !dead && toggleLights()}
        aria-label="Toggle lights"
      >
        <span style={iconStyle}>{state.lights_on ? "💡" : "🌙"}</span>
        <span>Light</span>
      </button>

      {/* Game */}
      <button
        style={btnStyle(dead || sleeping)}
        disabled={dead || sleeping}
        onClick={handleGame}
        aria-label="Play game"
      >
        <span style={iconStyle}>🎮</span>
        <span>Game</span>
      </button>

      {/* Medicine */}
      <button
        style={btnStyle(dead || !state.is_sick)}
        disabled={dead || !state.is_sick}
        onClick={() => !dead && state.is_sick && giveMedicine()}
        aria-label="Give medicine"
      >
        <span style={iconStyle}>💊</span>
        <span>Medicine</span>
      </button>

      {/* Bathroom / Clean */}
      <button
        style={btnStyle(dead || state.poop_count === 0)}
        disabled={dead || state.poop_count === 0}
        onClick={() => !dead && state.poop_count > 0 && cleanPoop()}
        aria-label="Clean poop"
      >
        <span style={iconStyle}>🚿</span>
        <span>Bathroom</span>
      </button>

      {/* Meter (info-only, shows stats) */}
      <button style={btnStyle(false)} disabled aria-label="Meter">
        <span style={iconStyle}>📊</span>
        <span>Meter</span>
      </button>

      {/* Discipline */}
      <button
        style={btnStyle(dead || state.pending_discipline_deadline === null)}
        disabled={dead || state.pending_discipline_deadline === null}
        onClick={() =>
          !dead && state.pending_discipline_deadline !== null && discipline()
        }
        aria-label="Discipline"
      >
        <span style={iconStyle}>📢</span>
        <span>Discipline</span>
      </button>

      {/* Attention (indicator) */}
      <button style={btnStyle(false)} disabled aria-label="Attention">
        <span style={iconStyle}>
          {state.pending_discipline_deadline !== null ? "❗" : "🔔"}
        </span>
        <span>Attention</span>
      </button>
    </div>
  );
}
