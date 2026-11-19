import { useEffect, useRef } from "react";
import "./CareControls.css";

// Each cell is one LCD pixel. The outlines leave space for the labels below.
const icons = {
  Feed: [
    "............",
    "....#..#....",
    "...#..#.....",
    "....#..#....",
    "............",
    ".##########.",
    ".#........#.",
    "..#......#..",
    "..##....##..",
    "....####....",
    "...######...",
    "............",
  ],
  Light: [
    ".....#......",
    "..#......#..",
    "....####....",
    "...#....#...",
    ".#.#....#.#.",
    "...#....#...",
    "....#..#....",
    "....####....",
    "....#..#....",
    "....####....",
    ".....##.....",
    "............",
  ],
  Game: [
    "............",
    "..########..",
    ".#........#.",
    ".#.##.....#.",
    ".#.##.....#.",
    ".#...##...#.",
    ".#...##...#.",
    ".#.....##.#.",
    ".#.....##.#.",
    ".#........#.",
    "..########..",
    "............",
  ],
  Medicine: [
    "............",
    "......###...",
    ".....#...#..",
    "....#.....#.",
    "...#......#.",
    "..###....#..",
    ".#####..#...",
    ".#######....",
    ".######.....",
    "..####......",
    "...##.......",
    "............",
  ],
  Clean: [
    "............",
    "..##........",
    ".#..#.......",
    ".#..##......",
    ".#..........",
    ".##########.",
    ".#........#.",
    "..#......#..",
    "..########..",
    "...#....#...",
    "............",
    "............",
  ],
  Stats: [
    "............",
    "........##..",
    "........##..",
    "........##..",
    ".....##.##..",
    ".....##.##..",
    "..##.##.##..",
    "..##.##.##..",
    "..##.##.##..",
    "............",
    ".##########.",
    "............",
  ],
  Discipline: [
    "............",
    ".....#..#...",
    "......#.....",
    "............",
    "...########.",
    "..#..#....#.",
    ".#....#####.",
    ".#.##.#.....",
    ".#.##.#.....",
    "..#..#......",
    "...##.......",
    "............",
  ],
  Attention: [
    "............",
    ".....##.....",
    "....#..#....",
    "...#....#...",
    "...#....#...",
    "...#....#...",
    "...#....#...",
    "..#......#..",
    ".##########.",
    "............",
    ".....##.....",
    "............",
  ],
};

export type CareAction = keyof typeof icons;
const rows: CareAction[][] = [
  ["Feed", "Light", "Game", "Medicine"],
  ["Clean", "Stats", "Discipline", "Attention"],
];

function PixelIcon({ action, lightsOn }: { action: CareAction; lightsOn: boolean }) {
  const ref = useRef<HTMLCanvasElement>(null);
  useEffect(() => {
    const ctx = ref.current?.getContext("2d");
    if (!ctx) return;
    ctx.clearRect(0, 0, 12, 12);
    ctx.fillStyle = "#2d3320";
    icons[action].forEach((row, y) => {
      for (let x = 0; x < row.length; x++) {
        if (row[x] !== "#") continue;
        if (action === "Light" && !lightsOn && (y < 2 || (y === 4 && (x === 1 || x === 10)))) continue;
        ctx.fillRect(x, y, 1, 1);
      }
    });
  }, [action, lightsOn]);
  return <canvas ref={ref} width={12} height={12} aria-hidden="true" />;
}

interface Props {
  onAction: (action: CareAction) => void;
  feedOpen: boolean;
  lightsOn: boolean;
  lightHint: string;
  attentionHint: string;
}

export default function CareControls({ onAction, feedOpen, lightsOn, lightHint, attentionHint }: Props) {
  return rows.map((actions, i) => (
    <div className={i === 0 ? "care-row top" : "care-row bottom"} key={actions[0]} role="group" aria-label={i === 0 ? "Pet care" : "Pet status and care"}>
      {actions.map(action => (
        <button
          type="button"
          key={action}
          onClick={() => onAction(action)}
          aria-expanded={action === "Feed" ? feedOpen : undefined}
          aria-pressed={action === "Light" ? lightsOn : undefined}
          title={action === "Light" ? lightHint : action === "Attention" ? attentionHint : action}
        >
          <PixelIcon action={action} lightsOn={lightsOn} />
          <span>{action}</span>
        </button>
      ))}
    </div>
  ));
}
