import type { PetState, LifeStage } from "../types";

interface PetDisplayProps {
  state: PetState;
}

const SPRITE_MAP: Record<LifeStage, Record<string, string>> = {
  Egg: { idle: "🥚" },
  Baby: { idle: "🐣", eating: "🐣🍼", sleeping: "🐣💤", happy: "🐣✨", sick: "🐣🤒", dead: "💀" },
  Child: { idle: "🐥", eating: "🐥🍽️", sleeping: "🐥💤", happy: "🐥✨", sick: "🐥🤒", dead: "💀" },
  Teen: { idle: "🐤", eating: "🐤🍽️", sleeping: "🐤💤", happy: "🐤✨", sick: "🐤🤒", dead: "💀" },
  Adult: { idle: "🐔", eating: "🐔🍽️", sleeping: "🐔💤", happy: "🐔✨", sick: "🐔🤒", dead: "💀" },
  Special: { idle: "👴", eating: "👴🍽️", sleeping: "👴💤", happy: "👴✨", sick: "👴🤒", dead: "💀" },
  Dead: { idle: "💀", dead: "💀" },
};

function getAnimationState(state: PetState): string {
  if (!state.is_alive) return "dead";
  if (state.is_sleeping) return "sleeping";
  if (state.is_sick) return "sick";
  if (state.happiness >= 4) return "happy";
  return "idle";
}

function getSprite(state: PetState): string {
  const animState = getAnimationState(state);
  const stageSprites = SPRITE_MAP[state.stage];
  return stageSprites?.[animState] ?? stageSprites?.idle ?? "❓";
}

function Hearts({ filled, max }: { filled: number; max: number }) {
  return (
    <span>
      {"❤️".repeat(filled)}
      {"🤍".repeat(max - filled)}
    </span>
  );
}

function DisciplineGauge({ value }: { value: number }) {
  return (
    <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
      <span style={{ fontSize: 12, minWidth: 70 }}>Discipline</span>
      <div
        style={{
          flex: 1,
          height: 12,
          background: "#444",
          borderRadius: 6,
          overflow: "hidden",
          maxWidth: 120,
        }}
      >
        <div
          style={{
            width: `${value}%`,
            height: "100%",
            background: value >= 75 ? "#4caf50" : value >= 50 ? "#ff9800" : "#f44336",
            borderRadius: 6,
            transition: "width 0.3s",
          }}
        />
      </div>
      <span style={{ fontSize: 12, minWidth: 30 }}>{value}%</span>
    </div>
  );
}

function PoopIndicators({ count }: { count: number }) {
  if (count === 0) return null;
  return <span>{"💩".repeat(count)}</span>;
}

export default function PetDisplay({ state }: PetDisplayProps) {
  const sprite = getSprite(state);

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        gap: 12,
        padding: 16,
        fontFamily: "monospace",
      }}
    >
      {/* Pet sprite area */}
      <div
        style={{
          fontSize: 64,
          lineHeight: 1,
          minHeight: 80,
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          position: "relative",
        }}
      >
        {sprite}
        {state.is_sick && state.is_alive && (
          <span style={{ position: "absolute", top: -8, right: -16, fontSize: 24 }}>🤒</span>
        )}
        {state.is_sleeping && state.is_alive && (
          <span style={{ position: "absolute", top: -8, left: -16, fontSize: 24 }}>💤</span>
        )}
      </div>

      {/* Poop indicators */}
      <div style={{ minHeight: 24 }}>
        <PoopIndicators count={state.poop_count} />
      </div>

      {/* Meters */}
      <div
        style={{
          display: "flex",
          flexDirection: "column",
          gap: 6,
          width: "100%",
          maxWidth: 260,
        }}
      >
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
          <span style={{ fontSize: 12 }}>Hunger</span>
          <Hearts filled={state.hunger} max={4} />
        </div>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
          <span style={{ fontSize: 12 }}>Happy</span>
          <Hearts filled={state.happiness} max={4} />
        </div>
        <DisciplineGauge value={state.discipline} />
      </div>

      {/* Stats */}
      <div
        style={{
          display: "flex",
          gap: 16,
          fontSize: 13,
          color: "#aaa",
        }}
      >
        <span>Age: {state.age}yr</span>
        <span>Weight: {state.weight}lb</span>
        <span>{state.character}</span>
      </div>
    </div>
  );
}
