import { useRef, useEffect, useState, useCallback } from "react";
import type { Choice, GameSession, PetState, RoundOutcome } from "../types";
import GameScreen from "./GameScreen";

export interface PetDisplayProps {
  state: PetState;
  feedMeal: () => Promise<void>;
  feedSnack: () => Promise<void>;
  discipline: () => Promise<void>;
  giveMedicine: () => Promise<void>;
  cleanPoop: () => Promise<void>;
  toggleLights: () => Promise<void>;
  startGame: () => Promise<GameSession>;
  playRound: (session: GameSession, choice: Choice) => Promise<RoundOutcome>;
}

// ── Pixel grid sprites (16x16 grids, 1 = dark pixel, 0 = off) ──────────────
type Sprite = number[];

const SPRITES: Record<string, Sprite> = {
  Egg: [
    0b0000001111000000, 0b0000111111110000, 0b0001110110111000, 0b0011111011011100,
    0b0011101101111100, 0b0011110110111100, 0b0011101101111100, 0b0011111011011100,
    0b0011110110111100, 0b0011101101111100, 0b0011111011011100, 0b0001110110111000,
    0b0000111111110000, 0b0000001111000000, 0b0000000000000000, 0b0000000000000000,
  ],
  Babytchi: [
    0b0000000000000000, 0b0000000110000000, 0b0000011111100000, 0b0000111111110000,
    0b0001111111111000, 0b0001101111011000, 0b0001111111111000, 0b0001111001111000,
    0b0001111111111000, 0b0000111111110000, 0b0000011111100000, 0b0000001111000000,
    0b0000010000100000, 0b0000010000100000, 0b0000000000000000, 0b0000000000000000,
  ],
  Marutchi: [
    0b0000000000000000, 0b0000011111100000, 0b0000111111110000, 0b0001111111111000,
    0b0011100110011100, 0b0011100110011100, 0b0011111111111100, 0b0011100000011100,
    0b0011110000111100, 0b0001111111111000, 0b0000111111110000, 0b0000011111100000,
    0b0000011001100000, 0b0000011001100000, 0b0000000000000000, 0b0000000000000000,
  ],
  Tamatchi: [
    0b0001100000011000, 0b0000110000110000, 0b0000111111110000, 0b0001111111111000,
    0b0001101111011000, 0b0001101111011000, 0b0001111111111000, 0b0001110000111000,
    0b0001111111111000, 0b0000111111110000, 0b0000011111100000, 0b0000001111000000,
    0b0000010000100000, 0b0000010000100000, 0b0000000000000000, 0b0000000000000000,
  ],
  Kuchitamatchi: [
    0b0000000000000000, 0b0000011111100000, 0b0000111111110000, 0b0001111111111000,
    0b0001101111011000, 0b0001111111111000, 0b0001111111111000, 0b0001111111111110,
    0b0001111111111110, 0b0001111111111000, 0b0000111111110000, 0b0000011111100000,
    0b0000010000100000, 0b0000010000100000, 0b0000000000000000, 0b0000000000000000,
  ],
  Mametchi: [
    0b0000000000000000, 0b0011111111111100, 0b0011111111111100, 0b0001111111111000,
    0b0001111111111000, 0b0001101111011000, 0b0001101111011000, 0b0001111111111000,
    0b0001110000111000, 0b0001111111111000, 0b0000111111110000, 0b0000011111100000,
    0b0000010000100000, 0b0000010000100000, 0b0000000000000000, 0b0000000000000000,
  ],
  Ginjirotchi: [
    0b0000000000000000, 0b0000010000100000, 0b0000111111110000, 0b0001111111111000,
    0b0011111111111100, 0b0011011111101100, 0b0011011111101100, 0b0011111111111100,
    0b0011111001111100, 0b0011111111111100, 0b0001111111111000, 0b0000111111110000,
    0b0000010000100000, 0b0000010000100000, 0b0000000000000000, 0b0000000000000000,
  ],
  Maskutchi: [
    0b0000000000000000, 0b0000111111110000, 0b0001111111111000, 0b0011111111111100,
    0b0011100011100100, 0b0011100011100100, 0b0011111111111100, 0b0011111111111100,
    0b0011110000111100, 0b0011111111111100, 0b0001111111111000, 0b0000111111110000,
    0b0000010000100000, 0b0000110000110000, 0b0000000000000000, 0b0000000000000000,
  ],
  Kuchipatchi: [
    0b0000000000000000, 0b0000011111100000, 0b0000111111110000, 0b0001111111111000,
    0b0011111111111100, 0b0011011111101100, 0b0011111111111100, 0b0011111111111100,
    0b0011111111111110, 0b0011111111111110, 0b0001111111111000, 0b0000111111110000,
    0b0000010000100000, 0b0000111001110000, 0b0000000000000000, 0b0000000000000000,
  ],
  Nyorotchi: [
    0b0000000000000000, 0b0000000000000000, 0b0000111110000000, 0b0001111111000000,
    0b0001101101000000, 0b0001111111000000, 0b0001110011000000, 0b0000111110000000,
    0b0000011100000000, 0b0000001110000000, 0b0000011111000000, 0b0000111111100000,
    0b0001111111110000, 0b0000111111100000, 0b0000011111000000, 0b0000000000000000,
  ],
  Tarakotchi: [
    0b0000100000010000, 0b0000010000100000, 0b0000111111110000, 0b0001111111111000,
    0b0011111111111100, 0b0011011111101100, 0b0011111111111100, 0b0011110000111100,
    0b0011111111111100, 0b0001111111111000, 0b0000111111110000, 0b0000010000100000,
    0b0000111001110000, 0b0000000000000000, 0b0000000000000000, 0b0000000000000000,
  ],
  Oyajitchi: [
    0b0000001111000000, 0b0000011111100000, 0b0000111111110000, 0b0001111111111000,
    0b0001101111011000, 0b0001101111011000, 0b0001111111111000, 0b0001100110011000,
    0b0001111001111000, 0b0001111111111000, 0b0000111111110000, 0b0000011111100000,
    0b0000010000100000, 0b0000010000100000, 0b0000000000000000, 0b0000000000000000,
  ],
  sleeping: [
    0b0000000000000000, 0b0000011111100000, 0b0000111111110000, 0b0001111111111000,
    0b0001110110111000, 0b0001111111111000, 0b0001111111111000, 0b0001111111111000,
    0b0000111111110000, 0b0000011111100000, 0b0000000000000000, 0b0000000000111000,
    0b0000000001000000, 0b0000000000011000, 0b0000000000000000, 0b0000000000000000,
  ],
  sick: [
    0b0000000000010000, 0b0000011111101000, 0b0000111111110000, 0b0001111111111000,
    0b0001010110101000, 0b0001111111111000, 0b0001111001111000, 0b0001111111111000,
    0b0000111111110000, 0b0000011111100000, 0b0000001111000000, 0b0000010000100000,
    0b0000010000100000, 0b0000000000000000, 0b0000000000000000, 0b0000000000000000,
  ],
  dead: [
    0b0000001111000000, 0b0000010000100000, 0b0000001111000000, 0b0000011111100000,
    0b0000111111110000, 0b0001111111111000, 0b0001010110101000, 0b0001111111111000,
    0b0001111001111000, 0b0001111111111000, 0b0000111111110000, 0b0000011111100000,
    0b0000010101010000, 0b0000001010100000, 0b0000000000000000, 0b0000000000000000,
  ],
  poop: [
    0b00000100, 0b00001000, 0b00010110, 0b00101001,
    0b01001001, 0b01001001, 0b00111110, 0b00000000,
  ],
};

const GLYPH_OK: number[] = [0b000001, 0b000010, 0b000100, 0b001000, 0b110000, 0b011000];
const GLYPH_X: number[] = [0b100001, 0b010010, 0b001100, 0b001100, 0b010010, 0b100001];
const GLYPH_PTR: number[] = [0b0010, 0b0110, 0b1110, 0b1111];

const GLYPH_BURGER: number[] = [
  0b01111110, 0b11111111, 0b11111111, 0b00000000,
  0b11111111, 0b00000000, 0b11111111, 0b11111111,
  0b01111110,
];
const GLYPH_CANDY: number[] = [
  0b01000010, 0b01100110, 0b01111110, 0b01111110,
  0b01111110, 0b01100110, 0b01000010, 0b00000000,
];

const FONT: Record<string, number[]> = {
  "0": [0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110],
  "1": [0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
  "2": [0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111],
  "3": [0b11111, 0b00010, 0b00100, 0b00010, 0b00001, 0b10001, 0b01110],
  "4": [0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010],
  "5": [0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110],
  "6": [0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110],
  "7": [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000],
  "8": [0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110],
  "9": [0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100],
  "/": [0b00001, 0b00010, 0b00010, 0b00100, 0b01000, 0b01000, 0b10000],
  "+": [0b00000, 0b00100, 0b00100, 0b11111, 0b00100, 0b00100, 0b00000],
  "W": [0b10001, 0b10001, 0b10001, 0b10101, 0b11111, 0b10101, 0b10001],
  "T": [0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100],
  "O": [0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
  "N": [0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001],
  "h": [0b00000, 0b01010, 0b11111, 0b11111, 0b01110, 0b00100, 0b00000],
  ".": [0b00000, 0b00000, 0b00000, 0b00000, 0b01100, 0b01100, 0b00000],
};

const CONFETTI_COLORS = ["#2d3320", "#556b2f", "#8a9a5b", "#b4bf91"];

function drawConfetti(
  ctx: CanvasRenderingContext2D, canvasW: number, px: number,
  frame: number, y: number, bandH: number, scale = 1,
) {
  const cell = px * scale;
  for (let i = 0; i < 14; i++) {
    const x = 2 * px + ((i * 37 + frame * 11) % (canvasW - 5 * px));
    const speed = 2 * px + ((i * 5) % 3) * px;
    const yy = y + ((frame * speed + ((i * 7) % 11) * px) % Math.max(bandH, px));
    const size = (((i * 3 + frame) % 2) + 1) * cell;
    ctx.fillStyle = CONFETTI_COLORS[i % CONFETTI_COLORS.length];
    ctx.fillRect(x, yy, size, size);
  }
}

function textWidth(text: string, px: number, scale = 1): number {
  return (text.length * (5 + 1) * px - px) * scale;
}

function drawText(
  ctx: CanvasRenderingContext2D, text: string,
  x: number, y: number, px: number, color: string, scale = 1,
) {
  const step = (5 + 1) * px * scale;
  const cell = px * scale;
  ctx.fillStyle = color;
  for (let ci = 0; ci < text.length; ci++) {
    const ch = text[ci];
    if (ch !== " ") {
      const glyph = FONT[ch];
      if (glyph) {
        for (let row = 0; row < glyph.length; row++) {
          for (let col = 0; col < 5; col++) {
            if ((glyph[row] >> (4 - col)) & 1) {
              ctx.fillRect(x + col * cell, y + row * cell, cell, cell);
            }
          }
        }
      }
    }
    x += step;
  }
}

// ── Icon strip sprites (8x8) ───────────────────────────────────────────────
const ICON_FEED: number[] = [0b00100100, 0b00100100, 0b01111110, 0b01000010, 0b01000010, 0b00111100, 0b00011000, 0b00000000];
const ICON_LIGHT: number[] = [0b00011000, 0b00100100, 0b01000010, 0b01011010, 0b01000010, 0b00100100, 0b00011000, 0b00000000];
const ICON_GAME: number[] = [0b00111100, 0b01000010, 0b10011001, 0b10000001, 0b10011001, 0b01000010, 0b00111100, 0b00000000];
const ICON_MED: number[] = [0b00011000, 0b00111100, 0b01111110, 0b00011000, 0b00011000, 0b00011000, 0b00011000, 0b00000000];
const ICON_BATH: number[] = [0b01000010, 0b00100100, 0b00011000, 0b01111110, 0b01111110, 0b01111110, 0b00111100, 0b00000000];
const ICON_METER: number[] = [0b01111110, 0b01000010, 0b01011010, 0b01010010, 0b01010010, 0b01000010, 0b01111110, 0b00000000];
const ICON_DISC: number[] = [0b00011000, 0b00111100, 0b01111110, 0b00011000, 0b00011000, 0b01100110, 0b01100110, 0b00000000];
const ICON_ATTN: number[] = [0b00011000, 0b00011000, 0b00011000, 0b00011000, 0b00011000, 0b00000000, 0b00011000, 0b00000000];

const TOP_ICONS = [ICON_FEED, ICON_LIGHT, ICON_GAME, ICON_MED];
const BOTTOM_ICONS = [ICON_BATH, ICON_METER, ICON_DISC, ICON_ATTN];

const TOP_LABELS = ["Feed", "Light", "Game", "Medicine"];
const BOTTOM_LABELS = ["Clean", "Stats", "Discipline", "Attention"];

const PX = 6;

// ── Rendering helpers ───────────────────────────────────────────────────────

function getSpriteKey(state: PetState): string {
  if (!state.is_alive) return "dead";
  if (state.stage === "Egg") return "Egg";
  if (state.is_sleeping) return "sleeping";
  if (state.is_sick) return "sick";
  return state.character;
}

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

/** Render an 8x8 icon row with highlight support. */
function drawIconRow(
  ctx: CanvasRenderingContext2D, icons: number[][], y: number,
  totalWidth: number, px: number, color: string, ghostColor: string,
  highlightIndex?: number,
) {
  const iconW = 8 * px;
  const spacing = (totalWidth - icons.length * iconW) / (icons.length + 1);
  icons.forEach((icon, i) => {
    const ix = spacing + i * (iconW + spacing);
    // Ghost background
    for (let row = 0; row < 8; row++) {
      for (let col = 0; col < 8; col++) {
        ctx.fillStyle = ghostColor;
        ctx.fillRect(ix + col * px, y + row * px, px, px);
      }
    }
    // Active pixels — highlighted icon gets inverted look
    const iconColor = i === highlightIndex ? "#556b2f" : color;
    drawSprite(ctx, icon, ix, y, 8, px, iconColor);
    // Highlight underline
    if (i === highlightIndex) {
      ctx.fillStyle = color;
      ctx.fillRect(ix, y + 8 * px - px, iconW, px);
    }
  });
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

/** Get the icon hit-test regions for a row of 4 icons. */
function getIconRegions(totalWidth: number, y: number, px: number) {
  const iconW = 8 * px;
  const spacing = (totalWidth - 4 * iconW) / 5;
  return Array.from({ length: 4 }, (_, i) => ({
    x: spacing + i * (iconW + spacing),
    y,
    w: iconW,
    h: 8 * px,
  }));
}

// ── Main component ──────────────────────────────────────────────────────────

export default function PetDisplay({
  state, feedMeal, feedSnack,
  discipline, giveMedicine, cleanPoop, toggleLights, startGame, playRound,
}: PetDisplayProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [hoverIcon, setHoverIcon] = useState<{ row: "top" | "bottom"; index: number } | null>(null);
  const [feedSubmenu, setFeedSubmenu] = useState(false);
  const [toast, setToast] = useState<string | null>(null);
  const [animFrame, setAnimFrame] = useState(0);
  const [actionAnim, setActionAnim] = useState(false);
  const [gameActive, setGameActive] = useState(false);
  const [revealAnim, setRevealAnim] = useState<{ dir: "Left" | "Right"; won: boolean; step: number } | null>(null);
  const [feedAnim, setFeedAnim] = useState<{ kind: "meal" | "snack"; step: number } | null>(null);
  const [gameScore, setGameScore] = useState<{ wins: number; happiness_gained: number; weight: number } | null>(null);
  const [celebrateFrame, setCelebrateFrame] = useState(0);
  const pendingScoreRef = useRef<{ wins: number; happiness_gained: number; weight: number } | null>(null);
  const lastLightsDeadlineRef = useRef<string | null>(null);

  const px = PX;
  const canvasW = 40 * px;
  const iconRowH = 8 * px;
  const spriteH = 16 * px;
  const heartRowH = 4 * px;
  const gap = 2 * px;
  const canvasH = iconRowH + gap + spriteH + gap + heartRowH + gap + heartRowH + gap + iconRowH;
  const bottomIconY = canvasH - iconRowH;

  // Idle animation timer — shift sprite position every ~1.2s
  useEffect(() => {
    if (!state.is_alive || state.is_sleeping) {
      setAnimFrame(0);
      return;
    }
    const interval = setInterval(() => {
      setAnimFrame(prev => (prev + 1) % 4);
    }, 1200);
    return () => clearInterval(interval);
  }, [state.is_alive, state.is_sleeping]);

  // Show a brief toast message
  const showToast = useCallback((msg: string) => {
    setToast(msg);
    setTimeout(() => setToast(null), 1500);
  }, []);

  const triggerActionAnim = useCallback((ms = 1000) => {
    setActionAnim(true);
    setTimeout(() => setActionAnim(false), ms);
  }, []);

  const handleReveal = useCallback((outcome: RoundOutcome) => {
    setRevealAnim({ dir: outcome.pet_choice, won: outcome.won, step: 0 });
    if (outcome.finished) {
      pendingScoreRef.current = {
        wins: outcome.wins,
        happiness_gained: outcome.happiness_gained,
        weight: outcome.weight,
      };
    }
  }, []);

  const handleGameStart = useCallback(() => {
    pendingScoreRef.current = null;
    setGameScore(null);
  }, []);

  const handleGameExit = useCallback(() => {
    pendingScoreRef.current = null;
    setGameScore(null);
    setGameActive(false);
  }, []);

  useEffect(() => {
    if (!revealAnim) return;
    if (revealAnim.step >= 26) {
      setRevealAnim(null);
      if (pendingScoreRef.current) {
        setGameScore(pendingScoreRef.current);
        pendingScoreRef.current = null;
      }
      return;
    }
    const t = setTimeout(() => {
      setRevealAnim(a => (a && a.step < 26 ? { ...a, step: a.step + 1 } : a));
    }, 60);
    return () => clearTimeout(t);
  }, [revealAnim]);

  useEffect(() => {
    if (!feedAnim) return;
    if (feedAnim.step >= 12) {
      setFeedAnim(null);
      return;
    }
    const t = setTimeout(() => {
      setFeedAnim(a => (a && a.step < 12 ? { ...a, step: a.step + 1 } : a));
    }, 60);
    return () => clearTimeout(t);
  }, [feedAnim]);

  useEffect(() => {
    if (!gameScore) {
      setCelebrateFrame(0);
      return;
    }
    const interval = setInterval(() => setCelebrateFrame(f => f + 1), 120);
    return () => clearInterval(interval);
  }, [gameScore]);

  useEffect(() => {
    if (!gameScore) return;
    const t = setTimeout(() => setGameScore(null), 4000);
    return () => clearTimeout(t);
  }, [gameScore]);

  const [bounce, setBounce] = useState(0);
  useEffect(() => {
    if (!actionAnim) {
      setBounce(0);
      return;
    }
    const interval = setInterval(() => {
      setBounce(prev => (prev + 1) % 8);
    }, 100);
    return () => clearInterval(interval);
  }, [actionAnim]);

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

  // Handle icon clicks
  const handleCanvasClick = useCallback(async (e: React.MouseEvent<HTMLCanvasElement>) => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const rect = canvas.getBoundingClientRect();
    // Use CSS dimensions for hit-test (not buffer size which includes DPR)
    const cx = (e.clientX - rect.left) * (canvasW / rect.width);
    const cy = (e.clientY - rect.top) * (canvasH / rect.height);

    // Top icon row
    const topRegions = getIconRegions(canvasW, 0, px);
    for (let i = 0; i < topRegions.length; i++) {
      const r = topRegions[i];
      if (cx >= r.x && cx <= r.x + r.w && cy >= r.y && cy <= r.y + r.h) {
        try {
          if (i === 0) { // Feed
            if (state.is_sleeping) {
              showToast("Pet is sleeping");
            } else {
              setFeedSubmenu(prev => !prev);
            }
          } else if (i === 1) { // Light
            await toggleLights();
          } else if (i === 2) { // Game
            if (gameActive) {
              pendingScoreRef.current = null;
              setGameScore(null);
              setGameActive(false);
            } else if (!state.is_sleeping) {
              setGameActive(true);
            } else {
              showToast("Pet is sleeping");
            }
          } else if (i === 3) { // Medicine
            await giveMedicine();
            triggerActionAnim();
            showToast("Medicine given");
          }
        } catch (err) { showToast(String(err)); }
        return;
      }
    }

    // Bottom icon row
    const bottomRegions = getIconRegions(canvasW, bottomIconY, px);
    for (let i = 0; i < bottomRegions.length; i++) {
      const r = bottomRegions[i];
      if (cx >= r.x && cx <= r.x + r.w && cy >= r.y && cy <= r.y + r.h) {
        try {
          if (i === 0) { // Bath/Clean
            await cleanPoop();
            showToast("Cleaned!");
          } else if (i === 1) { // Meter — no action, just info
            showToast(`❤${state.hunger}/4  😊${state.happiness}/4  WT:${state.weight}`);
          } else if (i === 2) { // Discipline
            await discipline();
            showToast("Disciplined!");
          } else if (i === 3) { // Attention — no action, just info
            const msgs: string[] = [];
            if (state.pending_lights_deadline) msgs.push("BEDTIME");
            if (state.is_sick) msgs.push("SICK");
            if (state.poop_count > 0) msgs.push("POOP");
            if (state.pending_discipline_deadline) msgs.push("DISCIPLINE");
            showToast(msgs.length ? msgs.join(" ") : sleepStatus.summary);
          }
        } catch (err) { showToast(String(err)); }
        return;
      }
    }

    // Click elsewhere closes feed submenu
    setFeedSubmenu(false);
  }, [canvasW, canvasH, bottomIconY, px, state, gameActive, toggleLights, giveMedicine, cleanPoop, discipline, showToast, triggerActionAnim, sleepStatus.summary]);

  // Handle hover for cursor + tooltip
  const handleCanvasMove = useCallback((e: React.MouseEvent<HTMLCanvasElement>) => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const rect = canvas.getBoundingClientRect();
    const cx = (e.clientX - rect.left) * (canvasW / rect.width);
    const cy = (e.clientY - rect.top) * (canvasH / rect.height);

    const topRegions = getIconRegions(canvasW, 0, px);
    for (let i = 0; i < topRegions.length; i++) {
      const r = topRegions[i];
      if (cx >= r.x && cx <= r.x + r.w && cy >= r.y && cy <= r.y + r.h) {
        setHoverIcon({ row: "top", index: i });
        canvas.style.cursor = "pointer";
        return;
      }
    }
    const bottomRegions = getIconRegions(canvasW, bottomIconY, px);
    for (let i = 0; i < bottomRegions.length; i++) {
      const r = bottomRegions[i];
      if (cx >= r.x && cx <= r.x + r.w && cy >= r.y && cy <= r.y + r.h) {
        setHoverIcon({ row: "bottom", index: i });
        canvas.style.cursor = "pointer";
        return;
      }
    }
    setHoverIcon(null);
    canvas.style.cursor = "default";
  }, [canvasW, canvasH, bottomIconY, px]);

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

    const pixel = "#2d3320";
    const ghost = "#b4bf91";

    const bg = "#c4cfa1";
    const s = revealAnim ? revealAnim.step : -1;
    ctx.fillStyle = bg;
    ctx.fillRect(0, 0, canvasW, canvasH);

    let y = 0;

    // Top icon row (with highlight)
    const topHi = hoverIcon?.row === "top" ? hoverIcon.index : undefined;
    drawIconRow(ctx, TOP_ICONS, y, canvasW, px, pixel, ghost, topHi);
    y += iconRowH + gap;

    // Pet sprite with idle movement
    const showScore = gameScore !== null;
    const spriteKey = getSpriteKey(state);
    const sprite = SPRITES[spriteKey] ?? SPRITES["Egg"];
    const baseSpriteX = (canvasW - 16 * px) / 2;
    // Idle horizontal offset: 0, +2px, 0, -2px
    const idleOffsets = [0, 2 * px, 0, -2 * px];
    const idleOn = state.is_alive && !state.is_sleeping && !revealAnim && !feedAnim;
    const offsetX = idleOn ? idleOffsets[animFrame] : 0;

    const feedStep = feedAnim ? feedAnim.step : -1;
    let feedGlyph: number[] | null = null;
    if (feedAnim && feedStep < 8) {
      feedGlyph = feedAnim.kind === "meal" ? GLYPH_BURGER : GLYPH_CANDY;
    }
    const feedJumpY = feedAnim
      ? (feedAnim.kind === "meal"
        ? [0, -px, -2 * px, -px, 0, -2 * px, -px, 0, 0, 0, 0, 0][feedStep]
        : [0, -px, 0, -px, 0, 0, 0, 0, 0, 0, 0, 0][feedStep])
      : 0;

    const DASH = 8 * px;
    const dirSign = revealAnim?.dir === "Left" ? -1 : 1;
    const easeOut = (p: number) => 1 - (1 - p) * (1 - p);
    let revealX = 0;
    let revealY = 0;
    let badge: { x: number; y: number; glyph: number[]; color: string } | null = null;
    let laneArrowX: number | null = null;
    if (revealAnim) {
      if (s <= 9) {
        const p = easeOut(s / 9);
        revealX = dirSign * DASH * p;
        revealY = s % 2 === 1 ? -px : 0;
      } else if (s <= 14) {
        revealX = dirSign * DASH;
        if (s % 2 === 0) {
          laneArrowX = revealAnim.dir === "Left" ? 1 * px : canvasW - 5 * px;
        }
      } else if (s <= 20) {
        revealX = dirSign * DASH;
        if (revealAnim.won) {
          revealY = [0, -px, -2 * px, -px, -2 * px, -px][s - 15];
        } else {
          revealY = [0, 2 * px, px, 2 * px, px, 0][s - 15];
        }
        const bx = revealX > 0
          ? baseSpriteX + revealX - 7 * px
          : baseSpriteX + revealX + 17 * px;
        badge = {
          x: bx,
          y: y + px,
          glyph: revealAnim.won ? GLYPH_OK : GLYPH_X,
          color: pixel,
        };
      } else {
        const p = 1 - (s - 21) / 4;
        revealX = dirSign * DASH * p;
      }
    }

    const actionBounceY = actionAnim ? [0, -px, 0, px][bounce % 4] : 0;
    let spriteY = y + actionBounceY + revealY + feedJumpY;
    const spriteX = baseSpriteX + offsetX + revealX;

    if (showScore && gameScore && gameScore.wins >= 3) {
      spriteY += [0, -px, -2 * px, -px, 0, 0][celebrateFrame % 6];
    }

    drawSprite(ctx, sprite, spriteX, spriteY, 16, px, pixel);

    if (feedGlyph) {
      const gy = y + 6 * px + (feedStep >= 5 ? 1 * px : 0);
      drawSprite(ctx, feedGlyph, spriteX - 9 * px, gy, 8, px, pixel);
    }
    if (feedAnim && feedStep >= 4 && feedStep <= 9) {
      const cx = spriteX + 8 * px;
      const cy = spriteY + 4 * px;
      for (let i = 0; i < 3; i++) {
        const dx = ((i * 5 + feedStep) % 18) - 9;
        const dy = ((i * 3 + feedStep) % 14) - 7;
        ctx.fillStyle = CONFETTI_COLORS[(feedStep + i) % CONFETTI_COLORS.length];
        ctx.fillRect(cx + dx * px, cy + dy * px, px, px);
      }
    }

    if (laneArrowX !== null) {
      drawSprite(ctx, GLYPH_PTR, laneArrowX, y + px, 4, px, pixel);
    }
    if (badge) {
      drawSprite(ctx, badge.glyph, badge.x, badge.y, 6, px, badge.color);
    }
    if (revealAnim && revealAnim.won && s >= 15 && s <= 20) {
      drawConfetti(ctx, canvasW, px, s, y, spriteH);
    }

    // Poop
    if (state.poop_count > 0 && state.is_alive) {
      const poopSprite = SPRITES["poop"];
      for (let i = 0; i < Math.min(state.poop_count, 2); i++) {
        const poopX = baseSpriteX + 16 * px + px;
        const poopY = y + spriteH - (i + 1) * 8 * px;
        if (poopSprite) drawSprite(ctx, poopSprite, poopX, poopY, 8, px, pixel);
      }
    }
    y += spriteH + gap;

    // Hunger hearts
    const scoreScale = 0.5;
    const heartsX = (canvasW - (4 * 4 * px + 3 * px)) / 2;
    if (showScore && gameScore) {
      const line = gameScore.wins >= 3 ? `WON ${gameScore.wins}/5` : `${gameScore.wins}/5`;
      drawText(ctx, line, (canvasW - textWidth(line, px, scoreScale)) / 2, y, px, pixel, scoreScale);
    } else {
      drawHearts(ctx, state.hunger, 4, heartsX, y, px, pixel, ghost);
    }
    y += heartRowH + gap;

    // Happiness hearts
    drawHearts(ctx, state.happiness, 4, heartsX, y, px, pixel, ghost);
    y += heartRowH + gap;

    if (showScore && gameScore && gameScore.wins >= 3) {
      drawConfetti(ctx, canvasW, px, celebrateFrame, iconRowH + gap, spriteH + gap + heartRowH + gap + heartRowH + gap);
    }

    // Bottom icon row (with highlight)
    const botHi = hoverIcon?.row === "bottom" ? hoverIcon.index : undefined;
    drawIconRow(ctx, BOTTOM_ICONS, y, canvasW, px, pixel, ghost, botHi);
  }, [state, canvasW, canvasH, px, hoverIcon, iconRowH, gap, spriteH, heartRowH, bottomIconY, animFrame, bounce, revealAnim, feedAnim, gameScore, celebrateFrame]);

  // Tooltip text
  const tooltipText = hoverIcon
    ? (
      hoverIcon.row === "top"
        ? (hoverIcon.index === 1 ? sleepStatus.summary : TOP_LABELS[hoverIcon.index])
        : (hoverIcon.index === 3 && state.pending_lights_deadline
          ? "Bedtime alert"
          : BOTTOM_LABELS[hoverIcon.index])
    )
    : null;

  return (
    <div style={containerStyle}>
      <div style={lcdFrameStyle}>
        <canvas
          ref={canvasRef}
          style={lcdStyle}
          onClick={handleCanvasClick}
          onMouseMove={handleCanvasMove}
          onMouseLeave={() => { setHoverIcon(null); }}
        />
        {/* Toast takes priority over tooltip */}
        {toast && (
          <div style={{
            ...toastStyle,
            ...(hoverIcon?.row === "bottom" || !hoverIcon
              ? { top: 4, bottom: "auto" }
              : { bottom: 4, top: "auto" }),
          }}>{toast}</div>
        )}
        {/* Tooltip */}
        {tooltipText && !toast && (
          <div style={{
            ...tooltipStyle,
            ...(hoverIcon?.row === "top"
              ? { top: "auto", bottom: 4 }
              : { bottom: "auto", top: 4 }),
          }}>{tooltipText}</div>
        )}
        {/* Feed submenu */}
        {feedSubmenu && (
          <div style={feedMenuStyle}>
            <button style={feedBtnStyle} onClick={async () => {
              setFeedSubmenu(false);
              try { await feedMeal(); setFeedAnim({ kind: "meal", step: 0 }); showToast("Meal fed!"); } catch (e) { showToast(String(e)); }
            }}>MEAL</button>
            <button style={feedBtnStyle} onClick={async () => {
              setFeedSubmenu(false);
              try { await feedSnack(); setFeedAnim({ kind: "snack", step: 0 }); showToast("Snack fed!"); } catch (e) { showToast(String(e)); }
            }}>SNACK</button>
          </div>
        )}
        {gameActive && (
          <GameScreen
            startGame={startGame}
            playRound={playRound}
            onClose={handleGameExit}
            onReveal={handleReveal}
            onGameStart={handleGameStart}
          />
        )}
      </div>
      <InfoPanel state={state} />
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
  const currentMinutes = now.getUTCHours() * 60 + now.getUTCMinutes();
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

const tooltipStyle: React.CSSProperties = {
  position: "absolute", bottom: 4, left: "50%", transform: "translateX(-50%)",
  background: "#333", color: "#fff", padding: "3px 8px", borderRadius: 3,
  fontSize: 10, fontFamily: "monospace", whiteSpace: "nowrap", pointerEvents: "none", zIndex: 20,
};

const toastStyle: React.CSSProperties = {
  position: "absolute", bottom: 4, left: "50%", transform: "translateX(-50%)",
  background: "#222", color: "#8a8", padding: "3px 8px", borderRadius: 3,
  fontSize: 10, fontFamily: "monospace", whiteSpace: "nowrap", pointerEvents: "none",
};

const feedMenuStyle: React.CSSProperties = {
  position: "absolute", top: 4, left: "50%", transform: "translateX(-50%)",
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
  whiteSpace: "nowrap",
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
