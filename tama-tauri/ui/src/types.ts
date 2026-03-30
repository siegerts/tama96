// TypeScript interfaces mirroring tama-core Rust structs

export type LifeStage =
  | "Egg"
  | "Baby"
  | "Child"
  | "Teen"
  | "Adult"
  | "Special"
  | "Dead";

export type Character =
  | "Babytchi"
  | "Marutchi"
  | "Tamatchi"
  | "Kuchitamatchi"
  | "Mametchi"
  | "Ginjirotchi"
  | "Maskutchi"
  | "Kuchipatchi"
  | "Nyorotchi"
  | "Tarakotchi"
  | "Oyajitchi";

export type TeenType = "Type1" | "Type2";

export type ActionType =
  | "FeedMeal"
  | "FeedSnack"
  | "PlayGame"
  | "Discipline"
  | "GiveMedicine"
  | "CleanPoop"
  | "ToggleLights"
  | "GetStatus";

export interface PetState {
  stage: LifeStage;
  character: Character;
  teen_type: TeenType | null;

  hunger: number;
  happiness: number;
  discipline: number;
  weight: number;
  age: number;

  care_mistakes: number;
  discipline_mistakes: number;

  poop_count: number;
  is_sick: boolean;
  sick_dose_count: number;
  is_sleeping: boolean;
  is_alive: boolean;
  lights_on: boolean;

  last_tick: string;
  birth_time: string;
  stage_start_time: string;
  last_poop_time: string;

  pending_care_deadline: string | null;
  pending_discipline_deadline: string | null;
  pending_lights_deadline: string | null;

  snack_count_since_last_tick: number;
}

export type Choice = "Left" | "Right";

export interface GameResult {
  rounds: number;
  wins: number;
  happiness_gained: number;
}

export interface ActionPermission {
  allowed: boolean;
  max_per_hour: number | null;
}

export interface ActionLogEntry {
  action: ActionType;
  timestamp: string;
}

export interface AgentPermissions {
  enabled: boolean;
  allowed_actions: Record<string, ActionPermission>;
  action_log: ActionLogEntry[];
}
