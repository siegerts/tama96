import { useCallback, useEffect, useState } from "react";
import type { Choice, GameSession, RoundOutcome } from "../types";

interface GameScreenProps {
  startGame: () => Promise<GameSession>;
  playRound: (session: GameSession, choice: Choice) => Promise<RoundOutcome>;
  onClose: () => void;
  onReveal: (outcome: RoundOutcome) => void;
  onGameStart: () => void;
}

const REVEAL_MS = 1700;

export default function GameScreen({ startGame, playRound, onClose, onReveal, onGameStart }: GameScreenProps) {
  const [session, setSession] = useState<GameSession | null>(null);
  const [phase, setPhase] = useState<"starting" | "pick" | "reveal" | "done">("starting");
  const [error, setError] = useState<string | null>(null);

  const beginGame = useCallback(async () => {
    onGameStart();
    setPhase("starting");
    setError(null);
    try {
      const s = await startGame();
      setSession(s);
      setPhase("pick");
    } catch (e) {
      setError(String(e));
      setPhase("pick");
    }
  }, [onGameStart, startGame]);

  useEffect(() => {
    beginGame();
  }, [beginGame]);

  const choose = async (choice: Choice) => {
    if (!session || phase !== "pick") return;
    setPhase("reveal");
    try {
      const outcome = await playRound(session, choice);
      onReveal(outcome);

      if (outcome.finished) {
        setTimeout(() => setPhase("done"), REVEAL_MS);
      } else {
        setSession({ ...session, round: outcome.round, wins: outcome.wins });
        setTimeout(() => setPhase("pick"), REVEAL_MS);
      }
    } catch (e) {
      setError(String(e));
      setPhase("pick");
    }
  };

  if (phase === "starting" || phase === "reveal") return null;

  return (
    <div style={menuStyle}>
      {error ? (
        <button style={btnStyle} onClick={onClose}>Done</button>
      ) : phase === "done" ? (
        <>
          <button style={btnStyle} onClick={beginGame}>Play Again</button>
          <button style={btnStyle} onClick={onClose}>Done</button>
        </>
      ) : (
        <>
          <button style={btnStyle} onClick={() => choose("Left")}>◀ LEFT</button>
          <button style={btnStyle} onClick={() => choose("Right")}>RIGHT ▶</button>
        </>
      )}
    </div>
  );
}

const menuStyle: React.CSSProperties = {
  position: "absolute",
  top: 4,
  left: "50%",
  transform: "translateX(-50%)",
  display: "flex",
  gap: 4,
  zIndex: 15,
};

const btnStyle: React.CSSProperties = {
  background: "#333",
  color: "#8a8",
  border: "1px solid #555",
  borderRadius: 3,
  padding: "4px 10px",
  cursor: "pointer",
  fontSize: 10,
  fontFamily: "monospace",
  letterSpacing: 1,
  whiteSpace: "nowrap",
};