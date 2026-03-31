import type { PetState } from "../types";

interface PetDisplayProps {
  state: PetState;
}

// ── Pixel grid sprites (16x16 grids, 1 = dark pixel, 0 = off) ──────────────
// Each sprite is a 16-element array of 16-bit numbers (MSB = leftmost pixel).

type Sprite = number[];

const SPRITES: Record<string, Sprite> = {
  // Egg — oval with diagonal stripe pattern (like the real P1 egg)
  Egg: [
    0b0000001111000000,
    0b0000111111110000,
    0b0001110110111000,
    0b0011111011011100,
    0b0011101101111100,
    0b0011110110111100,
    0b0011101101111100,
    0b0011111011011100,
    0b0011110110111100,
    0b0011101101111100,
    0b0011111011011100,
    0b0001110110111000,
    0b0000111111110000,
    0b0000001111000000,
    0b0000000000000000,
    0b0000000000000000,
  ],
  // Babytchi — tiny round body, two dot eyes, small mouth, stubby feet
  Babytchi: [
    0b0000000000000000,
    0b0000000110000000,
    0b0000011111100000,
    0b0000111111110000,
    0b0001111111111000,
    0b0001101111011000,
    0b0001111111111000,
    0b0001111001111000,
    0b0001111111111000,
    0b0000111111110000,
    0b0000011111100000,
    0b0000001111000000,
    0b0000010000100000,
    0b0000010000100000,
    0b0000000000000000,
    0b0000000000000000,
  ],
  // Marutchi — round body, big oval eyes, wide smile, small feet
  Marutchi: [
    0b0000000000000000,
    0b0000011111100000,
    0b0000111111110000,
    0b0001111111111000,
    0b0011100110011100,
    0b0011100110011100,
    0b0011111111111100,
    0b0011100000011100,
    0b0011110000111100,
    0b0001111111111000,
    0b0000111111110000,
    0b0000011111100000,
    0b0000011001100000,
    0b0000011001100000,
    0b0000000000000000,
    0b0000000000000000,
  ],
  // Tamatchi — tall oval body, pointy ear tufts, round eyes, smile
  Tamatchi: [
    0b0001100000011000,
    0b0000110000110000,
    0b0000111111110000,
    0b0001111111111000,
    0b0001101111011000,
    0b0001101111011000,
    0b0001111111111000,
    0b0001110000111000,
    0b0001111111111000,
    0b0000111111110000,
    0b0000011111100000,
    0b0000001111000000,
    0b0000010000100000,
    0b0000010000100000,
    0b0000000000000000,
    0b0000000000000000,
  ],
  // Kuchitamatchi — round body, beak/bill protruding, dot eyes
  Kuchitamatchi: [
    0b0000000000000000,
    0b0000011111100000,
    0b0000111111110000,
    0b0001111111111000,
    0b0001101111011000,
    0b0001111111111000,
    0b0001111111111000,
    0b0001111111111110,
    0b0001111111111110,
    0b0001111111111000,
    0b0000111111110000,
    0b0000011111100000,
    0b0000010000100000,
    0b0000010000100000,
    0b0000000000000000,
    0b0000000000000000,
  ],
  // Mametchi — ear-cap on top (wider than head), round eyes, happy mouth
  Mametchi: [
    0b0000000000000000,
    0b0011111111111100,
    0b0011111111111100,
    0b0001111111111000,
    0b0001111111111000,
    0b0001101111011000,
    0b0001101111011000,
    0b0001111111111000,
    0b0001110000111000,
    0b0001111111111000,
    0b0000111111110000,
    0b0000011111100000,
    0b0000010000100000,
    0b0000010000100000,
    0b0000000000000000,
    0b0000000000000000,
  ],
  // Ginjirotchi — round body, small horns/bumps on top, gentle face
  Ginjirotchi: [
    0b0000000000000000,
    0b0000010000100000,
    0b0000111111110000,
    0b0001111111111000,
    0b0011111111111100,
    0b0011011111101100,
    0b0011011111101100,
    0b0011111111111100,
    0b0011111001111100,
    0b0011111111111100,
    0b0001111111111000,
    0b0000111111110000,
    0b0000010000100000,
    0b0000010000100000,
    0b0000000000000000,
    0b0000000000000000,
  ],
  // Maskutchi — angular head, rectangular eyes (mask-like), stern mouth
  Maskutchi: [
    0b0000000000000000,
    0b0000111111110000,
    0b0001111111111000,
    0b0011111111111100,
    0b0011100011100100,
    0b0011100011100100,
    0b0011111111111100,
    0b0011111111111100,
    0b0011110000111100,
    0b0011111111111100,
    0b0001111111111000,
    0b0000111111110000,
    0b0000010000100000,
    0b0000110000110000,
    0b0000000000000000,
    0b0000000000000000,
  ],
  // Kuchipatchi — chubby round body, duck bill, happy eyes
  Kuchipatchi: [
    0b0000000000000000,
    0b0000011111100000,
    0b0000111111110000,
    0b0001111111111000,
    0b0011111111111100,
    0b0011011111101100,
    0b0011111111111100,
    0b0011111111111100,
    0b0011111111111110,
    0b0011111111111110,
    0b0001111111111000,
    0b0000111111110000,
    0b0000010000100000,
    0b0000111001110000,
    0b0000000000000000,
    0b0000000000000000,
  ],
  // Nyorotchi — snake/worm, long wavy body, small head with eyes
  Nyorotchi: [
    0b0000000000000000,
    0b0000000000000000,
    0b0000111110000000,
    0b0001111111000000,
    0b0001101101000000,
    0b0001111111000000,
    0b0001110011000000,
    0b0000111110000000,
    0b0000011100000000,
    0b0000001110000000,
    0b0000011111000000,
    0b0000111111100000,
    0b0001111111110000,
    0b0000111111100000,
    0b0000011111000000,
    0b0000000000000000,
  ],
  // Tarakotchi — antennae on top, round body, frowning mouth, wide feet
  Tarakotchi: [
    0b0000100000010000,
    0b0000010000100000,
    0b0000111111110000,
    0b0001111111111000,
    0b0011111111111100,
    0b0011011111101100,
    0b0011111111111100,
    0b0011110000111100,
    0b0011111111111100,
    0b0001111111111000,
    0b0000111111110000,
    0b0000010000100000,
    0b0000111001110000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
  ],
  // Oyajitchi — bald head, moustache, old man face
  Oyajitchi: [
    0b0000001111000000,
    0b0000011111100000,
    0b0000111111110000,
    0b0001111111111000,
    0b0001101111011000,
    0b0001101111011000,
    0b0001111111111000,
    0b0001100110011000,
    0b0001111001111000,
    0b0001111111111000,
    0b0000111111110000,
    0b0000011111100000,
    0b0000010000100000,
    0b0000010000100000,
    0b0000000000000000,
    0b0000000000000000,
  ],
  // Sleeping — generic sleeping pose, closed eyes, z's
  sleeping: [
    0b0000000000000000,
    0b0000011111100000,
    0b0000111111110000,
    0b0001111111111000,
    0b0001110110111000,
    0b0001111111111000,
    0b0001111111111000,
    0b0001111111111000,
    0b0000111111110000,
    0b0000011111100000,
    0b0000000000000000,
    0b0000000000111000,
    0b0000000001000000,
    0b0000000000011000,
    0b0000000000000000,
    0b0000000000000000,
  ],
  // Sick — sweat drop, X eyes, wavy mouth
  sick: [
    0b0000000000010000,
    0b0000011111101000,
    0b0000111111110000,
    0b0001111111111000,
    0b0001010110101000,
    0b0001111111111000,
    0b0001111001111000,
    0b0001111111111000,
    0b0000111111110000,
    0b0000011111100000,
    0b0000001111000000,
    0b0000010000100000,
    0b0000010000100000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
  ],
  // Dead — ghost/angel with halo, X eyes, wavy bottom
  dead: [
    0b0000001111000000,
    0b0000010000100000,
    0b0000001111000000,
    0b0000011111100000,
    0b0000111111110000,
    0b0001111111111000,
    0b0001010110101000,
    0b0001111111111000,
    0b0001111001111000,
    0b0001111111111000,
    0b0000111111110000,
    0b0000011111100000,
    0b0000010101010000,
    0b0000001010100000,
    0b0000000000000000,
    0b0000000000000000,
  ],
  // Poop — small swirl (8 rows, 8-bit)
  poop: [
    0b00000100,
    0b00001000,
    0b00010110,
    0b00101001,
    0b01001001,
    0b01001001,
    0b00111110,
    0b00000000,
  ],
};

// ── Icon strip sprites (8x8 for top/bottom icon rows) ───────────────────────

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

// ── Pixel size ──────────────────────────────────────────────────────────────

const PX = 6; // size of each "LCD pixel" in real CSS pixels

// ── Rendering helpers ───────────────────────────────────────────────────────

function getSpriteKey(state: PetState): string {
  if (!state.is_alive) return "dead";
  if (state.stage === "Egg") return "Egg";
  if (state.is_sleeping) return "sleeping";
  if (state.is_sick) return "sick";
  return state.character;
}

/** Render a pixel grid onto a canvas context. */
function drawSprite(
  ctx: CanvasRenderingContext2D,
  sprite: number[],
  x: number,
  y: number,
  bits: number,
  px: number,
  color: string,
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

/** Render an 8x8 icon row (4 icons spaced across the width). */
function drawIconRow(
  ctx: CanvasRenderingContext2D,
  icons: number[][],
  y: number,
  totalWidth: number,
  px: number,
  color: string,
  ghostColor: string,
) {
  const iconW = 8 * px;
  const spacing = (totalWidth - icons.length * iconW) / (icons.length + 1);
  icons.forEach((icon, i) => {
    const ix = spacing + i * (iconW + spacing);
    // Draw ghost (all pixels dim)
    for (let row = 0; row < 8; row++) {
      for (let col = 0; col < 8; col++) {
        ctx.fillStyle = ghostColor;
        ctx.fillRect(ix + col * px, y + row * px, px, px);
      }
    }
    // Draw active pixels on top
    drawSprite(ctx, icon, ix, y, 8, px, color);
  });
}

// ── Heart meter sprite (4x4 filled heart) ───────────────────────────────────
const HEART_FULL: number[] = [0b0110, 0b1111, 0b1111, 0b0110];
const HEART_EMPTY: number[] = [0b0110, 0b1001, 0b1001, 0b0110];

function drawHearts(
  ctx: CanvasRenderingContext2D,
  filled: number,
  max: number,
  x: number,
  y: number,
  px: number,
  activeColor: string,
  ghostColor: string,
) {
  for (let i = 0; i < max; i++) {
    const hx = x + i * (4 * px + px);
    const heart = i < filled ? HEART_FULL : HEART_EMPTY;
    const color = i < filled ? activeColor : ghostColor;
    drawSprite(ctx, heart, hx, y, 4, px, color);
  }
}

// ── Main component ──────────────────────────────────────────────────────────

import { useRef, useEffect } from "react";

export default function PetDisplay({ state }: PetDisplayProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  const px = PX;
  // LCD layout: 32 pixels wide (16 sprite + margins), rendered at PX scale
  // Total canvas: icon row (8px) + gap + sprite area (16px) + gap + hearts (4px) + gap + icon row (8px)
  const canvasW = 32 * px; // 32 "LCD pixels" wide
  const iconRowH = 8 * px;
  const spriteH = 16 * px;
  const heartRowH = 4 * px;
  const gap = 2 * px;
  const canvasH = iconRowH + gap + spriteH + gap + heartRowH + gap + heartRowH + gap + iconRowH;

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    // LCD colors
    const bg = "#c4cfa1";
    const pixel = "#2d3320";
    const ghost = "#b4bf91";

    // Clear
    ctx.fillStyle = bg;
    ctx.fillRect(0, 0, canvasW, canvasH);

    let y = 0;

    // Top icon row
    drawIconRow(ctx, TOP_ICONS, y, canvasW, px, pixel, ghost);
    y += iconRowH + gap;

    // Pet sprite (centered in 32px-wide area, sprite is 16px)
    const spriteKey = getSpriteKey(state);
    const sprite = SPRITES[spriteKey] ?? SPRITES["Egg"];
    const spriteX = (canvasW - 16 * px) / 2;
    drawSprite(ctx, sprite, spriteX, y, 16, px, pixel);

    // Draw poop next to pet if present
    if (state.poop_count > 0 && state.is_alive) {
      const poopSprite = SPRITES["poop"];
      for (let i = 0; i < Math.min(state.poop_count, 2); i++) {
        const poopX = spriteX + 16 * px + px;
        const poopY = y + spriteH - (i + 1) * 8 * px;
        if (poopSprite) {
          drawSprite(ctx, poopSprite, poopX, poopY, 8, px, pixel);
        }
      }
    }

    y += spriteH + gap;

    // Hunger hearts row
    const heartsX = (canvasW - (4 * 4 * px + 3 * px)) / 2;
    drawHearts(ctx, state.hunger, 4, heartsX, y, px, pixel, ghost);
    y += heartRowH + gap;

    // Happiness hearts row
    drawHearts(ctx, state.happiness, 4, heartsX, y, px, pixel, ghost);
    y += heartRowH + gap;

    // Bottom icon row
    drawIconRow(ctx, BOTTOM_ICONS, y, canvasW, px, pixel, ghost);
  }, [state, canvasW, canvasH, px]);

  return (
    <div style={containerStyle}>
      <canvas
        ref={canvasRef}
        width={canvasW}
        height={canvasH}
        style={lcdStyle}
      />
      {/* Info below the LCD */}
      <div style={infoStyle}>
        <span>{state.character}</span>
        <span>AGE {state.age}</span>
        <span>WT {state.weight}</span>
        {state.is_sick && <span>SICK</span>}
        {state.is_sleeping && <span>ZZZ</span>}
      </div>
    </div>
  );
}

const containerStyle: React.CSSProperties = {
  display: "flex",
  flexDirection: "column",
  alignItems: "center",
  gap: 4,
  flex: 1,
  width: "100%",
  padding: "4px 0",
};

const lcdStyle: React.CSSProperties = {
  border: "3px solid #6b7353",
  borderRadius: 4,
  boxShadow: "inset 0 0 12px rgba(0,0,0,0.1), 0 2px 8px rgba(0,0,0,0.3)",
  imageRendering: "pixelated",
  display: "block",
};

const infoStyle: React.CSSProperties = {
  display: "flex",
  gap: 10,
  fontSize: 10,
  fontFamily: "monospace",
  color: "#555",
  letterSpacing: 1,
  textTransform: "uppercase",
};
