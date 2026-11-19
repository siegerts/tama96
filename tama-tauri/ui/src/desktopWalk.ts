import { invoke } from "@tauri-apps/api/core";
import { emergenceDuration, emergenceRenderer } from "../../../assets/pet-emergence";

export interface WalkStatus { supported: boolean; away: boolean }

export async function walkOnDesktop(
  canvas: HTMLCanvasElement, rows: string[], character: string, birthTime: string,
) {
  const cell = document.createElement("canvas");
  cell.width = cell.height = 256;
  const atlas = document.createElement("canvas");
  atlas.width = cell.width * 8;
  atlas.height = cell.height;
  const ctx = atlas.getContext("2d");
  if (!ctx) throw new Error("Could not draw the walking pet.");
  const draw = emergenceRenderer(cell, rows, character);
  // Side-profile characters already face left in the original sprite.
  const angle = character === "Kuchipatchi" ? -0.66 : 0.24;
  for (let i = 0; i < 8; i++) {
    draw(emergenceDuration, angle, (2 * Math.PI * i) / 8);
    ctx.drawImage(cell, i * cell.width, 0);
  }
  const rect = canvas.getBoundingClientRect();
  await invoke("start_desktop_walk", {
    departure: {
      character, birth_time: birthTime, atlas: atlas.toDataURL("image/png").split(",")[1],
      x: rect.left + rect.width / 2, y: rect.top + rect.height / 2,
      size: Math.min(rect.width, rect.height),
    },
  });
}
