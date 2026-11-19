import { useEffect, useRef, useState } from "react";
import artwork from "../../../../assets/pet-frames.json";
import { animateEmergence } from "../../../../assets/pet-emergence";
import type { Choice, GameProgress, PetState } from "../types";
import { walkOnDesktop } from "../desktopWalk";
import "./GuessGame.css";

interface Props {
  state: PetState;
  guessDirection: (choice: Choice, displayed: PetState) => Promise<PetState>;
  canRoam: boolean;
  onDepart: () => void;
  onClose: () => void;
}

export default function GuessGame({ state, guessDirection, canRoam, onDepart, onClose }: Props) {
  const dlg = useRef<HTMLDialogElement>(null);
  const canvas = useRef<HTMLCanvasElement>(null);
  const locked = useRef(false);
  const [busy, setBusy] = useState(false);
  const [view, setView] = useState<"play" | "score" | "3d">("play");
  const [error, setError] = useState("");
  const [replay, setReplay] = useState(0);
  const [finished, setFinished] = useState(false);
  const [score, setScore] = useState<GameProgress | null>(null);
  const [celebrate, setCelebrate] = useState(false);
  const active = useRef(true);
  const name = state.stage === "Egg" ? "Egg" : state.character;
  const rows = artwork[name].rest;
  const game = view === "score" && score ? score : state.game;
  const round = game.turn % 5;
  const showLook = view === "score" || round > 0;
  const walkUnavailable = state.is_sleeping ? "Your pet is sleeping." : "";
  const gameUnavailable = walkUnavailable || (state.is_sick ? "Your pet needs medicine." : "");
  useEffect(() => {
    const dialog = dlg.current!;
    active.current = true;
    dialog.showModal();
    return () => {
      active.current = false;
      dialog.close();
    };
  }, []);

  useEffect(() => {
    const element = canvas.current;
    if (!element) return;
    if (view === "3d") return animateEmergence(element, rows, name, () => setFinished(true));
    const ctx = element.getContext("2d");
    if (!ctx) return;
    const size = element.width;
    const pixel = Math.floor(size / 18);
    ctx.clearRect(0, 0, size, size);
    ctx.fillStyle = "#303a25";
    rows.forEach((row, y) =>
      [...row].forEach((cell, x) => {
        if (cell === "#")
          ctx.fillRect(
            (1 + (showLook && game.look === "Right" ? 15 - x : x)) * pixel,
            (y + 1) * pixel,
            pixel,
            pixel,
          );
      }),
    );
  }, [rows, name, view, replay, showLook, game.look]);

  useEffect(() => {
    if (!celebrate || view !== "score") return;
    const timer = window.setTimeout(() => {
      setCelebrate(false);
      setView("3d");
    }, 1050);
    return () => window.clearTimeout(timer);
  }, [celebrate, view]);

  async function choose(choice: Choice) {
    if (locked.current || view !== "play" || gameUnavailable) return;
    locked.current = true;
    setBusy(true);
    setError("");
    try {
      const next = await guessDirection(choice, state);
      if (active.current && next.game.turn % 5 === 0) {
        setFinished(false);
        setScore(next.game);
        setCelebrate(game.stars < 3 && next.game.stars >= 3);
        setView("score");
      }
    } catch (e) {
      const messages: Record<string, string> = {
        TurnAlreadyChanged: "Another view played that turn. Try the next one.",
        PetIsSleeping: "Your pet is sleeping.",
        PetIsSick: "Your pet needs medicine.",
        PetIsDead: "Your pet can no longer play.",
      };
      if (active.current) setError(messages[String(e)] ?? String(e));
    } finally {
      locked.current = false;
      if (active.current) setBusy(false);
    }
  }
  async function depart() {
    if (!canvas.current || locked.current) return;
    locked.current = true;
    setBusy(true);
    setError("");
    try {
      await walkOnDesktop(canvas.current, rows, name, state.birth_time);
      onDepart();
    } catch (e) {
      if (active.current) setError(String(e));
    } finally {
      locked.current = false;
      if (active.current) setBusy(false);
    }
  }

  function emerge() {
    setError("");
    setCelebrate(false);
    setFinished(false);
    setReplay((n) => n + 1);
    setView("3d");
  }

  return (
    <dialog
      ref={dlg}
      className={`guess-dialog ${view === "3d" ? "emerged" : ""}`}
      aria-labelledby="guess-heading"
      onCancel={onClose}
      onKeyDown={(e) => {
        if (view === "play" && (e.key === "ArrowLeft" || e.key === "ArrowRight")) {
          e.preventDefault();
          if (!e.repeat) void choose(e.key === "ArrowLeft" ? "Left" : "Right");
        }
      }}
    >
      <header>
        <h2 id="guess-heading">{view === "3d" ? name : "Left or right?"}</h2>
        <button className="game-close" onClick={onClose} aria-label="Close game">
          ×
        </button>
      </header>
      {view === "3d" ? (
        <>
          <canvas
            className="sculpture"
            ref={canvas}
            width={480}
            height={480}
            aria-label={`A three-dimensional pixel sculpture of ${name}`}
          />
          <p className="game-note" role="status">
            {error || (canRoam && walkUnavailable) || (finished ? (
              <>
                <span>3D form unlocked</span> · Drag to turn
              </>
            ) : (
              "Growing out of the LCD…"
            ))}
          </p>
          {canRoam && (
            <button className="walk-launch" onClick={depart} disabled={!finished || busy || !!walkUnavailable}>
              {busy ? "Stepping out…" : "Walk on desktop ↗"}
            </button>
          )}
          <div className="game-actions">
            <button disabled={busy} onClick={emerge}>Replay</button>
            <button onClick={() => setView(score ? "score" : "play")}>
              Back to game
            </button>
          </div>
        </>
      ) : (
        <>
          <div className="guess-lcd">
            <div className="game-counter">
              <span>{view === "score" ? "Final score" : `Round ${round + 1} / 5`}</span>
              <span>★ {game.stars}</span>
            </div>
            <canvas ref={canvas} width={108} height={108} aria-label={name} />
            <strong aria-live="polite">
              {view === "score"
                ? `${game.hits} / 5 matches`
                : showLook
                  ? `${game.look === "Left" ? "← Left" : "Right →"} · ${game.matched ? "Match!" : "Miss"}`
                  : "Which way will I look?"}
            </strong>
          </div>
          {view === "play" ? (
            <div className="game-actions">
              <button disabled={busy || !!gameUnavailable} onClick={() => choose("Left")}>
                ← Left
              </button>
              <button disabled={busy || !!gameUnavailable} onClick={() => choose("Right")}>
                Right →
              </button>
            </div>
          ) : (
            <div className="game-actions">
              <button
                onClick={() => {
                  setCelebrate(false);
                  setScore(null);
                  setView("play");
                }}
              >
                Play again
              </button>
            </div>
          )}
          <p className="game-note" role="status">
            {error ||
              gameUnavailable ||
              (view === "score"
                ? `${game.look === "Left" ? "← Left" : "Right →"} · ${game.matched ? "Match!" : "Miss"} · +${game.hits} stars`
                : `One star per match · Best ${game.best}/5`)}
          </p>
          {game.stars >= 3 ? (
            <button className="form-link" onClick={emerge}>
              View 3D form ↗
            </button>
          ) : (
            <p className="unlock-meter">
              3D form <progress max={3} value={game.stars} />
              {game.stars}/3 ★
            </p>
          )}
        </>
      )}
    </dialog>
  );
}
