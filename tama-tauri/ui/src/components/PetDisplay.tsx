import { useRef, useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { WalkStatus } from "../desktopWalk";
import type { PetState, Choice } from "../types";
import { idleMotion, petFrame } from "../petSprites";
import GuessGame from "./GuessGame";
import CareControls, { type CareAction } from "./CareControls";

export interface PetDisplayProps {
  state: PetState;
  feedMeal: () => Promise<void>;
  feedSnack: () => Promise<void>;
  guessDirection: (choice: Choice, displayed: PetState) => Promise<PetState>;
  discipline: () => Promise<void>;
  giveMedicine: () => Promise<void>;
  cleanPoop: () => Promise<void>;
  toggleLights: () => Promise<void>;
}

const POOP = [
  0b00000100, 0b00001000, 0b00010110, 0b00101001,
  0b01001001, 0b01001001, 0b00111110, 0b00000000,
];

const PX = 6;

// ── Rendering helpers ───────────────────────────────────────────────────────

function drawSprite(
  ctx: CanvasRenderingContext2D, sprite: number[],
  x: number, y: number, bits: number, px: number, color: string,
) {
  ctx.fillStyle = color;
  for (let row = 0; row < sprite.length; row++) {
    for (let col = 0; col < bits; col++) {
      if ((sprite[row] >> (bits - 1 - col)) & 1) {
        ctx.fillRect(x + col * px, y + row * px, px, px);
      }
    }
  }
}

const HEART_FULL: number[] = [0b0110, 0b1111, 0b1111, 0b0110];
const HEART_EMPTY: number[] = [0b0110, 0b1001, 0b1001, 0b0110];

function drawHearts(
  ctx: CanvasRenderingContext2D, filled: number, max: number,
  x: number, y: number, px: number, activeColor: string, ghostColor: string,
) {
  for (let i = 0; i < max; i++) {
    const hx = x + i * (4 * px + px);
    const heart = i < filled ? HEART_FULL : HEART_EMPTY;
    const color = i < filled ? activeColor : ghostColor;
    drawSprite(ctx, heart, hx, y, 4, px, color);
  }
}

// ── Main component ──────────────────────────────────────────────────────────

export default function PetDisplay({
  state, feedMeal, feedSnack, guessDirection,
  discipline, giveMedicine, cleanPoop, toggleLights,
}: PetDisplayProps) {
  const [gameOpen, setGameOpen] = useState(false);
  const [walk, setWalk] = useState<WalkStatus>({ supported: false, away: false });
  useEffect(() => {
    let mounted = true;
    let received = false;
    let unlisten: (() => void) | undefined;
    async function connect() {
      const off = await listen<boolean>("desktop-walk-changed", ({ payload }) => {
        if (!mounted) return;
        received = true;
        setWalk({ supported: true, away: payload });
      });
      if (!mounted) { off(); return; }
      unlisten = off;
      const status = await invoke<WalkStatus>("desktop_walk_status");
      if (mounted && !received && status) setWalk(status);
    }
    void connect().catch(() => {});
    return () => { mounted = false; unlisten?.(); };
  }, []);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [feedSubmenu, setFeedSubmenu] = useState(false);
  const [toast, setToast] = useState<string | null>(null);
  const [animFrame, setAnimFrame] = useState(0);
  const [feeding, setFeeding] = useState(false);
  const previousWeight = useRef(state.weight);
  const motion = idleMotion(state);
  const lastLightsDeadlineRef = useRef<string | null>(null);

  const px = PX;
  const canvasW = 40 * px;
  const iconRowH = 8 * px;
  const spriteH = 16 * px;
  const heartRowH = 4 * px;
  const gap = 2 * px;
  const canvasH = iconRowH + gap + spriteH + gap + heartRowH + gap + heartRowH + gap + iconRowH;

  useEffect(() => {
    if (motion === "still") {
      setAnimFrame(0);
      return;
    }
    const delay = motion === "gesture" && animFrame % 2 === 1 ? 300 : 1200;
    const timer = window.setTimeout(() => setAnimFrame((animFrame + 1) % 4), delay);
    return () => window.clearTimeout(timer);
  }, [motion, animFrame]);

  // Meals and snacks both add weight, even at full hearts. Observing the
  // snapshot also catches food given through the terminal or an agent.
  useEffect(() => {
    const ate = state.weight > previousWeight.current;
    previousWeight.current = state.weight;
    const showBite = ate && state.character === "Kuchipatchi" && !state.is_sleeping && !state.is_sick;
    setFeeding(showBite);
    if (showBite) {
      const timer = window.setTimeout(() => setFeeding(false), 450);
      return () => window.clearTimeout(timer);
    }
  }, [state.weight, state.character, state.is_sleeping, state.is_sick]);

  // Show a brief toast message
  const showToast = useCallback((msg: string) => {
    setToast(msg);
    setTimeout(() => setToast(null), 1500);
  }, []);

  useEffect(() => {
    if (
      state.pending_lights_deadline
      && state.pending_lights_deadline !== lastLightsDeadlineRef.current
    ) {
      showToast("Bedtime! Lights off in 15m");
    }
    lastLightsDeadlineRef.current = state.pending_lights_deadline;
  }, [state.pending_lights_deadline, showToast]);

  const sleepStatus = getSleepStatus(state);

  async function handleAction(action: CareAction) {
    if (action === "Feed") {
      setFeedSubmenu(open => !open);
      return;
    }
    setFeedSubmenu(false);
    try {
      switch (action) {
        case "Light":
          await toggleLights();
          break;
        case "Game":
          if (walk.away) showToast("Bring your pet home to play");
          else setGameOpen(true);
          break;
        case "Medicine":
          await giveMedicine();
          showToast("Medicine given");
          break;
        case "Clean":
          await cleanPoop();
          showToast("Cleaned!");
          break;
        case "Stats":
          showToast(`❤${state.hunger}/4  😊${state.happiness}/4  WT:${state.weight}`);
          break;
        case "Discipline":
          await discipline();
          showToast("Disciplined!");
          break;
        case "Attention": {
          const messages = [];
          if (state.pending_lights_deadline) messages.push("BEDTIME");
          if (state.is_sick) messages.push("SICK");
          if (state.poop_count > 0) messages.push("POOP");
          if (state.pending_discipline_deadline) messages.push("DISCIPLINE");
          showToast(messages.join(" ") || sleepStatus.summary);
          break;
        }
      }
    } catch (e) {
      showToast(String(e));
    }
  }

  // Draw canvas
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    // Handle HiDPI: set canvas buffer size to match device pixels
    const dpr = window.devicePixelRatio || 1;
    canvas.width = canvasW * dpr;
    canvas.height = canvasH * dpr;
    canvas.style.width = `${canvasW}px`;
    canvas.style.height = `${canvasH}px`;
    ctx.scale(dpr, dpr);

    const bg = "#c4cfa1";
    const pixel = "#2d3320";
    const ghost = "#b4bf91";

    ctx.fillStyle = bg;
    ctx.fillRect(0, 0, canvasW, canvasH);

    let y = iconRowH + gap;
    const baseSpriteX = (canvasW - 16 * px) / 2;
    const offsetX = motion === "drift" ? [0, 2, 0, -2][animFrame] * px : 0;
    if (!walk.away) drawSprite(ctx, petFrame(state, animFrame, feeding), baseSpriteX + offsetX, y, 16, px, pixel);

    // Poop
    if (state.poop_count > 0 && state.is_alive) {
      for (let i = 0; i < Math.min(state.poop_count, 2); i++) {
        const poopX = baseSpriteX + 16 * px + px;
        const poopY = y + spriteH - (i + 1) * 8 * px;
        drawSprite(ctx, POOP, poopX, poopY, 8, px, pixel);
      }
    }
    y += spriteH + gap;

    // Hunger hearts
    const heartsX = (canvasW - (4 * 4 * px + 3 * px)) / 2;
    drawHearts(ctx, state.hunger, 4, heartsX, y, px, pixel, ghost);
    y += heartRowH + gap;

    // Happiness hearts
    drawHearts(ctx, state.happiness, 4, heartsX, y, px, pixel, ghost);
  }, [state, canvasW, canvasH, px, iconRowH, gap, spriteH, heartRowH, animFrame, motion, feeding, walk.away]);

  return (
    <div style={containerStyle}>
      <div className="pet-lcd" style={lcdFrameStyle}>
        <canvas
          ref={canvasRef}
          style={lcdStyle}
          onClick={() => setFeedSubmenu(false)}
          aria-label={`${state.character}, hunger ${state.hunger} of 4, happiness ${state.happiness} of 4`}
        />
        <div className="heart-readout" tabIndex={0} role="group" aria-label={`Hunger ${state.hunger} of 4, Happiness ${state.happiness} of 4`}>
          <span>Hunger</span><span>Happiness</span>
        </div>
        {walk.away && (
          <div className="walk-home">
            <span>Out on the desktop</span>
            <button onClick={() => invoke("bring_pet_home").catch(e => showToast(String(e)))}>Bring home</button>
          </div>
        )}
        <CareControls
          onAction={handleAction}
          feedOpen={feedSubmenu}
          lightsOn={state.lights_on}
          lightHint={sleepStatus.summary}
          attentionHint={state.pending_lights_deadline ? "Bedtime alert" : "Attention"}
        />
        {toast && <div role="status" style={toastStyle}>{toast}</div>}
        {/* Feed submenu */}
        {feedSubmenu && (
          <div style={feedMenuStyle}>
            <button style={feedBtnStyle} onClick={async () => {
              setFeedSubmenu(false);
              try { await feedMeal(); showToast("Meal fed!"); } catch (e) { showToast(String(e)); }
            }}>MEAL</button>
            <button style={feedBtnStyle} onClick={async () => {
              setFeedSubmenu(false);
              try { await feedSnack(); showToast("Snack fed!"); } catch (e) { showToast(String(e)); }
            }}>SNACK</button>
          </div>
        )}
      </div>
      <InfoPanel state={state} />
      {gameOpen && <GuessGame state={state} guessDirection={guessDirection} canRoam={walk.supported} onDepart={() => setGameOpen(false)} onClose={() => setGameOpen(false)} />}
    </div>
  );
}

// ── Character stats lookup (mirrors Rust CharacterStats) ────────────────────
const CHAR_STATS: Record<string, {
  hungerDecay: number; happyDecay: number; poopInterval: number;
  sleepHour: number; wakeHour: number; maxLifespan: number;
}> = {
  Babytchi: { hungerDecay: 30, happyDecay: 30, poopInterval: 60, sleepHour: 20, wakeHour: 8, maxLifespan: 0 },
  Marutchi: { hungerDecay: 20, happyDecay: 25, poopInterval: 45, sleepHour: 20, wakeHour: 9, maxLifespan: 0 },
  Tamatchi: { hungerDecay: 18, happyDecay: 22, poopInterval: 40, sleepHour: 21, wakeHour: 9, maxLifespan: 0 },
  Kuchitamatchi: { hungerDecay: 16, happyDecay: 20, poopInterval: 35, sleepHour: 21, wakeHour: 9, maxLifespan: 0 },
  Mametchi: { hungerDecay: 12, happyDecay: 15, poopInterval: 30, sleepHour: 22, wakeHour: 9, maxLifespan: 16 },
  Ginjirotchi: { hungerDecay: 14, happyDecay: 17, poopInterval: 35, sleepHour: 22, wakeHour: 9, maxLifespan: 12 },
  Maskutchi: { hungerDecay: 10, happyDecay: 12, poopInterval: 25, sleepHour: 22, wakeHour: 9, maxLifespan: 16 },
  Kuchipatchi: { hungerDecay: 8, happyDecay: 10, poopInterval: 20, sleepHour: 21, wakeHour: 9, maxLifespan: 6 },
  Nyorotchi: { hungerDecay: 6, happyDecay: 7, poopInterval: 20, sleepHour: 21, wakeHour: 9, maxLifespan: 3 },
  Tarakotchi: { hungerDecay: 7, happyDecay: 8, poopInterval: 20, sleepHour: 21, wakeHour: 9, maxLifespan: 4 },
  Oyajitchi: { hungerDecay: 10, happyDecay: 12, poopInterval: 25, sleepHour: 22, wakeHour: 9, maxLifespan: 16 },
};

// Evolution thresholds (minutes in stage)
const EVOLUTION_INFO: Record<string, { ageThreshold?: number; minutesThreshold?: number; label: string }> = {
  Egg: { minutesThreshold: 5, label: "Hatch" },
  Baby: { minutesThreshold: 65, label: "Child" },
  Child: { ageThreshold: 3, label: "Teen" },
  Teen: { ageThreshold: 6, label: "Adult" },
};

type SleepTone = "neutral" | "normal" | "alert" | "sleep";

function minutesUntilHour(hour: number, now: Date): number {
  const currentMinutes = now.getHours() * 60 + now.getMinutes();
  const targetMinutes = hour * 60;
  let diff = targetMinutes - currentMinutes;
  if (diff < 0) diff += 24 * 60;
  return diff;
}

function formatMinutes(mins: number): string {
  if (mins <= 0) return "soon";
  if (mins < 60) return `${Math.round(mins)}m`;
  const h = Math.floor(mins / 60);
  const m = Math.round(mins % 60);
  return m > 0 ? `${h}h${m}m` : `${h}h`;
}

function formatCountdown(prefix: string, mins: number): string {
  return mins <= 0 ? `${prefix} now` : `${prefix} ${formatMinutes(mins)}`;
}

function getSleepStatus(
  state: PetState,
  nowMs: number = Date.now(),
): { summary: string; hint: string; detail: string; tone: SleepTone } {
  if (state.stage === "Egg") {
    return {
      summary: "No bedtime yet",
      hint: "Sleep starts after hatch",
      detail: "Age starts after hatch",
      tone: "neutral",
    };
  }

  const stats = CHAR_STATS[state.character];
  if (!stats) {
    return {
      summary: "Sleep unknown",
      hint: "Sleep timing unavailable",
      detail: "Age +1 on wake",
      tone: "neutral",
    };
  }

  const now = new Date(nowMs);
  const wakeIn = minutesUntilHour(stats.wakeHour, now);
  const bedtimeIn = minutesUntilHour(stats.sleepHour, now);

  if (state.is_sleeping) {
    return {
      summary: formatCountdown("Wake", wakeIn),
      hint: `Sleeping now. ${formatCountdown("Wake", wakeIn)}`,
      detail: "Age +1 on wake",
      tone: "sleep",
    };
  }

  if (state.pending_lights_deadline) {
    const remaining = (new Date(state.pending_lights_deadline).getTime() - nowMs) / 60000;
    return {
      summary: remaining <= 0 ? "Lights off now" : `Lights off ${formatMinutes(remaining)}`,
      hint: remaining <= 0
        ? "Bedtime now. Turn lights off."
        : `Bedtime now. Lights off in ${formatMinutes(remaining)}`,
      detail: "Age +1 on wake",
      tone: "alert",
    };
  }

  return {
    summary: formatCountdown("Bed", bedtimeIn),
    hint: `Next bedtime in ${formatMinutes(bedtimeIn)}`,
    detail: "Age +1 on wake",
    tone: "normal",
  };
}

function InfoPanel({ state }: { state: PetState }) {
  const stats = CHAR_STATS[state.character];
  const now = Date.now();
  const lastTick = new Date(state.last_tick).getTime();
  const stageStart = new Date(state.stage_start_time).getTime();
  const lastPoop = new Date(state.last_poop_time).getTime();
  const elapsedSinceTick = (now - lastTick) / 60000; // minutes
  const sleepStatus = getSleepStatus(state, now);

  // Time until next hunger drop
  const hungerRemaining = stats ? Math.max(0, stats.hungerDecay - elapsedSinceTick) : 0;
  // Time until next poop
  const poopElapsed = (now - lastPoop) / 60000;
  const poopRemaining = stats ? Math.max(0, stats.poopInterval - poopElapsed) : 0;

  // Time until next evolution
  let evoText = "";
  const evoInfo = EVOLUTION_INFO[state.stage];
  if (evoInfo) {
    if (evoInfo.minutesThreshold) {
      const elapsed = (now - stageStart) / 60000;
      const remaining = Math.max(0, evoInfo.minutesThreshold - elapsed);
      evoText = `${evoInfo.label} in ${formatMinutes(remaining)}`;
    } else if (evoInfo.ageThreshold) {
      const daysLeft = evoInfo.ageThreshold - state.age;
      evoText = daysLeft > 0 ? `${evoInfo.label} in ${daysLeft}d` : `${evoInfo.label} soon`;
    }
  } else if (stats?.maxLifespan && stats.maxLifespan > 0) {
    const daysLeft = stats.maxLifespan - state.age;
    evoText = daysLeft > 0 ? `Lifespan: ${daysLeft}d left` : "";
  }

  // Pending deadlines
  const deadlines: string[] = [];
  if (state.pending_care_deadline) {
    const remaining = (new Date(state.pending_care_deadline).getTime() - now) / 60000;
    if (remaining > 0) deadlines.push(`Care: ${formatMinutes(remaining)}`);
  }
  if (state.pending_discipline_deadline) {
    const remaining = (new Date(state.pending_discipline_deadline).getTime() - now) / 60000;
    if (remaining > 0) deadlines.push(`Disc: ${formatMinutes(remaining)}`);
  }
  if (state.pending_lights_deadline) {
    const remaining = (new Date(state.pending_lights_deadline).getTime() - now) / 60000;
    if (remaining > 0) deadlines.push(`Lights: ${formatMinutes(remaining)}`);
  }

  // Build tooltip lines
  const tipLines: string[] = [
    sleepStatus.hint,
    sleepStatus.detail,
    `-hunger ${formatMinutes(hungerRemaining)}`,
    `+poop ${formatMinutes(poopRemaining)}`,
  ];
  if (evoText) tipLines.push(evoText);
  if (state.is_sick) tipLines.push(`SICK (${state.sick_dose_count}/2 doses)`);
  if (state.is_sleeping) tipLines.push("ZZZ");
  tipLines.push(...deadlines);

  const [showTip, setShowTip] = useState(false);

  return (
    <div
      style={infoPanelStyle}
      onMouseEnter={() => setShowTip(true)}
      onMouseLeave={() => setShowTip(false)}
    >
      <div style={infoMainRow}>
        <span>{state.character}</span>
        <span>age {state.age}</span>
        <span>wt {state.weight}</span>
      </div>
      <div style={infoMetaRow}>
        <span style={infoChipStyle(sleepStatus.tone)}>{sleepStatus.summary}</span>
        <span style={infoChipStyle("neutral")}>{sleepStatus.detail}</span>
      </div>
      {showTip && (
        <div style={infoTipStyle}>
          {tipLines.map((line, i) => <div key={i}>{line}</div>)}
        </div>
      )}
    </div>
  );
}

const containerStyle: React.CSSProperties = {
  display: "flex", flexDirection: "column", alignItems: "center",
  gap: 4, width: "100%", padding: "4px 0",
};

const lcdFrameStyle: React.CSSProperties = {
  position: "relative",
  padding: 3,
  borderRadius: 9,
  background: "linear-gradient(180deg, rgba(74, 96, 72, 0.45), rgba(53, 69, 48, 0.34))",
  boxShadow: [
    "inset 0 3px 8px rgba(26, 38, 25, 0.42)",
    "inset 0 -2px 4px rgba(255, 255, 255, 0.16)",
    "0 1px 0 rgba(255, 255, 255, 0.08)",
  ].join(", "),
};

const lcdStyle: React.CSSProperties = {
  border: "2px solid #747e5d",
  borderRadius: 6,
  boxShadow: [
    "inset 0 1px 0 rgba(255, 255, 255, 0.18)",
    "inset 0 -14px 20px rgba(78, 87, 50, 0.16)",
    "inset 10px 0 14px rgba(255, 255, 255, 0.04)",
    "inset -10px 0 14px rgba(52, 61, 32, 0.05)",
  ].join(", "),
  imageRendering: "pixelated",
  display: "block",
};

const toastStyle: React.CSSProperties = {
  position: "absolute", bottom: 55, left: "50%", transform: "translateX(-50%)",
  background: "#222", color: "#8a8", padding: "3px 8px", borderRadius: 3,
  fontSize: 10, fontFamily: "monospace", whiteSpace: "nowrap", pointerEvents: "none",
};

const feedMenuStyle: React.CSSProperties = {
  position: "absolute", top: 51, left: "50%", transform: "translateX(-50%)",
  display: "flex", gap: 4, background: "#222e", border: "1px solid #555",
  borderRadius: 4, padding: 4, zIndex: 15,
};

const feedBtnStyle: React.CSSProperties = {
  background: "#333", color: "#8a8", border: "1px solid #555", borderRadius: 3,
  padding: "4px 10px", cursor: "pointer", fontSize: 10, fontFamily: "monospace",
  letterSpacing: 1,
};

const infoPanelStyle: React.CSSProperties = {
  position: "relative", display: "flex", flexDirection: "column",
  alignItems: "center", width: "100%", maxWidth: 200, padding: "4px 0",
  cursor: "default",
};

const infoMainRow: React.CSSProperties = {
  display: "flex", justifyContent: "center", gap: 10,
  fontSize: 12, fontFamily: "monospace", color: "#444",
  letterSpacing: 0.5, fontVariantNumeric: "tabular-nums",
};

const infoMetaRow: React.CSSProperties = {
  display: "flex", justifyContent: "center", flexWrap: "wrap",
  gap: 6, marginTop: 4, maxWidth: "100%",
};

const infoChipStyle = (tone: SleepTone): React.CSSProperties => ({
  background: tone === "alert"
    ? "#3d2616"
    : tone === "sleep"
      ? "#223047"
      : "#00000012",
  color: tone === "alert"
    ? "#f2c48f"
    : tone === "sleep"
      ? "#c2d7ff"
      : "#4a4a4a",
  borderRadius: 999,
  padding: "2px 7px",
  fontSize: 10,
  fontFamily: "monospace",
  letterSpacing: 0.4,
  fontVariantNumeric: "tabular-nums",
});

const infoTipStyle: React.CSSProperties = {
  position: "absolute", bottom: "100%", left: "50%", transform: "translateX(-50%)",
  background: "#333", color: "#ccc", padding: "6px 10px", borderRadius: 4,
  fontSize: 10, fontFamily: "monospace", whiteSpace: "nowrap", zIndex: 20,
  lineHeight: 1.6, marginBottom: 2, fontVariantNumeric: "tabular-nums",
};
