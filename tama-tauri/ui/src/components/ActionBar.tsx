import { useState, useRef } from "react";
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

function Tooltip({ text, children }: { text: string; children: React.ReactNode }) {
  const [show, setShow] = useState(false);
  const timeoutRef = useRef<number | undefined>(undefined);

  const handleMouseEnter = () => {
    timeoutRef.current = window.setTimeout(() => setShow(true), 500);
  };
  const handleMouseLeave = () => {
    if (timeoutRef.current) clearTimeout(timeoutRef.current);
    setShow(false);
  };

  return (
    <div style={{ position: "relative" }} onMouseEnter={handleMouseEnter} onMouseLeave={handleMouseLeave}>
      {children}
      {show && (
        <div style={tooltipStyle}>
          {text}
        </div>
      )}
    </div>
  );
}

const tooltipStyle: React.CSSProperties = {
  position: "absolute",
  bottom: "100%",
  left: "50%",
  transform: "translateX(-50%)",
  marginBottom: 4,
  background: "#333",
  color: "#fff",
  padding: "4px 8px",
  borderRadius: 3,
  fontSize: 10,
  whiteSpace: "nowrap",
  zIndex: 20,
  pointerEvents: "none",
};

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
  const [error, setError] = useState<string | null>(null);

  const dead = !state.is_alive;
  const sleeping = state.is_sleeping;

  const btnStyle = (disabled: boolean, active?: boolean): React.CSSProperties => ({
    display: "flex",
    flexDirection: "column",
    alignItems: "center",
    justifyContent: "center",
    padding: "6px 4px",
    border: active ? "2px solid #8a8" : "1px solid #555",
    borderRadius: 3,
    background: disabled ? "#1a1a1a" : active ? "#2a3a2a" : "#222",
    color: disabled ? "#444" : "#8a8",
    cursor: disabled ? "not-allowed" : "pointer",
    fontSize: 10,
    fontFamily: "monospace",
    letterSpacing: 1,
    minWidth: 52,
    minHeight: 36,
    opacity: disabled ? 0.4 : 1,
    textTransform: "uppercase" as const,
    position: "relative" as const,
  });

  const handleFeedClick = () => {
    if (dead || sleeping) return;
    setFeedOpen((o) => !o);
  };

  const handleMeal = async () => {
    setFeedOpen(false);
    setError(null);
    try { await feedMeal(); } catch (e) { setError(String(e)); }
  };

  const handleSnack = async () => {
    setFeedOpen(false);
    setError(null);
    try { await feedSnack(); } catch (e) { setError(String(e)); }
  };

  const handleGame = async () => {
    if (dead || sleeping) return;
    await playGame(randomMoves());
  };

  const hasDisciplineCall = state.pending_discipline_deadline !== null;

  // Show error message if any
  const errorText = error ? (
    <div style={{ width: "100%", textAlign: "center", color: "#f66", fontSize: 10, padding: "2px 0" }}>
      {error}
    </div>
  ) : null;

  return (
    <div style={barStyle}>
      {errorText}
      {/* Feed */}
      <Tooltip text="Give food to your pet">
        <div style={{ position: "relative" }}>
          <button
            style={btnStyle(dead || sleeping)}
            disabled={dead || sleeping}
            onClick={handleFeedClick}
            aria-label="Feed"
            aria-expanded={feedOpen}
          >
            FEED
          </button>
          {feedOpen && !dead && !sleeping && (
            <div style={submenuStyle}>
              <Tooltip text="Restores 1 hunger, +1 weight">
                <button style={btnStyle(false)} onClick={handleMeal} aria-label="Feed meal">
                  MEAL
                </button>
              </Tooltip>
              <Tooltip text="Restores 1 happiness, +2 weight">
                <button style={btnStyle(false)} onClick={handleSnack} aria-label="Feed snack">
                  SNCK
                </button>
              </Tooltip>
            </div>
          )}
        </div>
      </Tooltip>

      {/* Light */}
      <Tooltip text="Toggle day/night mode">
        <button
          style={btnStyle(dead, !state.lights_on)}
          disabled={dead}
          onClick={() => !dead && toggleLights()}
          aria-label="Toggle lights"
        >
          {state.lights_on ? "LITE" : "DARK"}
        </button>
      </Tooltip>

      {/* Game */}
      <Tooltip text="Play a guessing game for happiness">
        <button
          style={btnStyle(dead || sleeping)}
          disabled={dead || sleeping}
          onClick={handleGame}
          aria-label="Play game"
        >
          GAME
        </button>
      </Tooltip>

      {/* Medicine */}
      <Tooltip text="Cure sickness (needs 2 doses)">
        <button
          style={btnStyle(dead || !state.is_sick)}
          disabled={dead || !state.is_sick}
          onClick={() => !dead && state.is_sick && giveMedicine()}
          aria-label="Give medicine"
        >
          MED
        </button>
      </Tooltip>

      {/* Bathroom */}
      <Tooltip text="Clean up poop">
        <button
          style={btnStyle(dead || state.poop_count === 0)}
          disabled={dead || state.poop_count === 0}
          onClick={() => !dead && state.poop_count > 0 && cleanPoop()}
          aria-label="Clean poop"
        >
          BATH
        </button>
      </Tooltip>

      {/* Discipline */}
      <Tooltip text="Scold when pet needs discipline">
        <button
          style={btnStyle(dead || !hasDisciplineCall, hasDisciplineCall)}
          disabled={dead || !hasDisciplineCall}
          onClick={() => !dead && hasDisciplineCall && discipline()}
          aria-label="Discipline"
        >
          {hasDisciplineCall ? "DISC!" : "DISC"}
        </button>
      </Tooltip>
    </div>
  );
}

const barStyle: React.CSSProperties = {
  display: "flex",
  flexWrap: "wrap",
  justifyContent: "center",
  gap: 3,
  padding: "4px 2px",
  maxWidth: 280,
  width: "100%",
};

const submenuStyle: React.CSSProperties = {
  position: "absolute",
  bottom: "100%",
  left: "50%",
  transform: "translateX(-50%)",
  marginBottom: 4,
  display: "flex",
  gap: 4,
  background: "#111",
  border: "1px solid #555",
  borderRadius: 3,
  padding: 3,
  zIndex: 10,
};
