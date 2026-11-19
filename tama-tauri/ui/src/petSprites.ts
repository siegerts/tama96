import artwork from "../../../assets/pet-frames.json";
import type { PetState } from "./types";

type Poses<Row> = { rest: Row[]; gesture?: Row[]; sleeping?: Row[]; sick?: Row[] };

function decode(rows: string[]): number[] {
  return rows.map(row => parseInt(row.replace(/\./g, "0").replace(/#/g, "1"), 2));
}

const frames: Record<string, Poses<number>> = Object.fromEntries(
  Object.entries(artwork).map(([name, data]: [string, Poses<string>]) => [name, {
    rest: decode(data.rest),
    gesture: data.gesture && decode(data.gesture),
    sleeping: data.sleeping && decode(data.sleeping),
    sick: data.sick && decode(data.sick),
  }]),
);

export function idleMotion(state: PetState) {
  if (!state.is_alive || state.is_sleeping || state.is_sick) return "still";
  if (state.stage === "Egg") return "drift";
  switch (state.character) {
    case "Babytchi":
    case "Tamatchi": return "gesture";
    case "Marutchi": return state.happiness >= 3 ? "gesture" : "still";
    case "Kuchipatchi": return "still";
    default: return "drift";
  }
}

export function petFrame(state: PetState, beat: number, feeding: boolean) {
  const poses = frames[state.character] ?? frames.Egg;
  if (!state.is_alive) return frames.dead.rest;
  if (state.stage === "Egg") return frames.Egg.rest;
  if (state.is_sleeping) return poses.sleeping ?? frames.sleeping.rest;
  if (state.is_sick) return poses.sick ?? frames.sick.rest;

  const gesture = state.character === "Kuchipatchi" ? feeding : idleMotion(state) === "gesture" && beat % 2 === 1;
  return gesture && poses.gesture ? poses.gesture : poses.rest;
}
