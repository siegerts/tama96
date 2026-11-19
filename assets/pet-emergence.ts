// The same 16 × 16 ink mask drives the opening frame and the raised surface.
type Point = [number, number, number];
type Face = { corners: Point[]; normal: Point; lightNormal: Point; color: Color; ink: boolean };
type Color = [number, number, number];

const LCD: Color = [196, 207, 161];
const INK: Color = [45, 51, 32];
const NIGHT: Color = [22, 36, 43];
const ease = (n: number) => {
  const t = Math.max(0, Math.min(1, n));
  return t * t * (3 - 2 * t);
};
const mix = (a: Color, b: Color, t: number): Color => a.map((v, i) => v + (b[i] - v) * t) as Color;
const rgb = (c: Color) => `rgb(${c.map(Math.round).join(",")})`;

function buildSculpture(rows: string[], character: string) {
  const size = 16;
  const outside = new Set<number>();
  const index = (x: number, y: number) => (y + 1) * 18 + x + 1;
  const neighbors = [
    [0, -1],
    [0, 1],
    [-1, 0],
    [1, 0],
  ];
  const queue = [[-1, -1]];
  for (let i = 0; i < queue.length; i++) {
    const [x, y] = queue[i];
    for (const [dx, dy] of neighbors) {
      const nx = x + dx,
        ny = y + dy,
        k = index(nx, ny);
      if (nx < -1 || ny < -1 || nx > size || ny > size || outside.has(k) || rows[ny]?.[nx] === "#")
        continue;
      outside.add(k);
      queue.push([nx, ny]);
    }
  }
  const depth = new Map<number, number>();
  const contour: number[][] = [];
  for (let y = 0; y < size; y++)
    for (let x = 0; x < size; x++) {
      if (
        !outside.has(index(x, y)) &&
        neighbors.some(([dx, dy]) => outside.has(index(x + dx, y + dy)))
      )
        contour.push([x, y]);
    }
  const colors: Record<string, Color> = {
    Kuchipatchi: [119, 218, 157],
    Mametchi: [245, 208, 101],
    Tamatchi: [114, 193, 217],
    Marutchi: [249, 173, 137],
    Babytchi: [151, 143, 211],
  };
  const body = colors[character] ?? [136, 201, 174];
  let left = 16,
    right = 0,
    top = 16,
    bottom = 0;
  for (let y = 0; y < size; y++)
    for (let x = 0; x < size; x++) {
      if (outside.has(index(x, y))) continue;
      const distance = Math.min(...contour.map(([cx, cy]) => Math.hypot(x - cx, y - cy)));
      let radius = Math.min(4.5, 0.75 + 1.6 * Math.sqrt(distance));
      if (character === "Kuchipatchi") {
        const oval = 1 - ((x - 7.3) / 8.4) ** 2 - ((y - 8) / 7.2) ** 2;
        radius = 0.7 + 4.1 * Math.sqrt(Math.max(0, oval));
        if (x <= 3 && y >= 6 && y <= 10) radius = y === 8 ? 2.4 : 3.2;
        if (y === 14) radius = 1.5;
      }
      depth.set(index(x, y), radius);
      left = Math.min(left, x);
      right = Math.max(right, x + 1);
      top = Math.min(top, y);
      bottom = Math.max(bottom, y + 1);
    }
  const solid = rows.join("").split("#").length - 1 > depth.size * 0.68;
  const faces: Face[] = [];
  const originX = (left + right) / 2,
    originY = (top + bottom) / 2;
  for (let y = 0; y < size; y++)
    for (let x = 0; x < size; x++) {
      const z = depth.get(index(x, y));
      if (!z) continue;
      const x0 = x - originX,
        y0 = y - originY,
        x1 = x0 + 1,
        y1 = y0 + 1;
      const ink = rows[y][x] === "#";
      const border = [-1, 0, 1].some((dx) =>
        [-1, 0, 1].some((dy) => outside.has(index(x + dx, y + dy))),
      );
      let faceColor: Color = mix(body, [236, 255, 206], 0.24 * (1 - y / 16));
      if (solid && !ink) faceColor = [250, 239, 197];
      if (!solid && ink && !border) faceColor = [36, 50, 51];
      if (character === "Kuchipatchi") {
        if (x <= 3 && y >= 6 && y <= 10) faceColor = y === 8 ? [66, 102, 63] : [190, 212, 112];
        if (y === 14) faceColor = [85, 153, 113];
        if (y === 5 && (x === 4 || x === 7)) faceColor = [33, 45, 44];
        if (y === 7 && x === 7) faceColor = [244, 173, 140];
      }
      const sample = (dx: number, dy: number) => depth.get(index(x + dx, y + dy)) ?? 0;
      const bevel: Point = [
        (sample(-1, 0) - sample(1, 0)) * 0.55,
        (sample(0, -1) - sample(0, 1)) * 0.55,
        1,
      ];
      const add = (corners: Point[], normal: Point, color: Color, dark = false) => {
        const surface = normal[2] > 0 ? bevel : normal;
        const length = Math.hypot(...surface);
        const lightNormal = surface.map((component) => component / length) as Point;
        faces.push({ corners, normal, lightNormal, color, ink: dark });
      };
      add(
        [
          [x0, y0, z],
          [x1, y0, z],
          [x1, y1, z],
          [x0, y1, z],
        ],
        [0, 0, 1],
        faceColor,
        ink,
      );
      add(
        [
          [x1, y0, -z],
          [x0, y0, -z],
          [x0, y1, -z],
          [x1, y1, -z],
        ],
        [0, 0, -1],
        body,
      );
      for (const [dx, dy] of neighbors) {
        const adjacent = depth.get(index(x + dx, y + dy)) ?? 0;
        if (adjacent >= z) continue;
        const ax = dx === 1 ? x1 : x0,
          ay = dy === 1 ? y1 : y0;
        const bx = dx === 0 ? x1 : ax,
          by = dy === 0 ? y1 : ay;
        const tint = mix(faceColor, dy < 0 ? [246, 255, 213] : [33, 83, 83], dy < 0 ? 0.28 : 0.17);
        for (const [near, far] of adjacent
          ? [
              [adjacent, z],
              [-z, -adjacent],
            ]
          : [[-z, z]]) {
          add(
            [
              [ax, ay, near],
              [bx, by, near],
              [bx, by, far],
              [ax, ay, far],
            ],
            [dx, dy, 0],
            tint,
          );
        }
      }
    }
  return { faces, originX, originY, footY: bottom - originY };
}

export const emergenceDuration = 3300;

export function emergenceRenderer(canvas: HTMLCanvasElement, rows: string[], character: string) {
  const ctx = canvas.getContext("2d");
  const { faces, originX, originY, footY } = buildSculpture(rows, character);
  return (milliseconds: number, orbit = 0, stride?: number) => {
    if (!ctx) return;
    const width = canvas.width,
      height = canvas.height;
    const t = Math.max(0, Math.min(1, milliseconds / emergenceDuration));
    const grow = ease((t - 0.12) / 0.55),
      light = ease((t - 0.1) / 0.45);
    const angle = (0.44 + 2 * Math.PI) * ease((t - 0.18) / 0.79) + orbit;
    const tilt = -0.16 * grow - 0.17 * Math.sin(Math.PI * grow);
    const co = Math.cos(angle),
      si = Math.sin(angle),
      ct = Math.cos(tilt),
      st = Math.sin(tilt);
    const unit = Math.min(width, height) * (0.032 + 0.012 * grow);
    const cx = width / 2,
      cy = height * (0.49 + 0.035 * grow - 0.055 * Math.sin(Math.PI * grow));
    if (stride === undefined) {
      ctx.fillStyle = rgb(mix(LCD, NIGHT, light));
      ctx.fillRect(0, 0, width, height);
    } else {
      ctx.clearRect(0, 0, width, height);
    }
    if (t <= 0.12) {
      ctx.fillStyle = rgb(INK);
      rows.forEach((row, y) =>
        [...row].forEach((cell, x) => {
          if (cell === "#")
            ctx.fillRect(
              cx + (x - originX) * unit,
              cy + (y - originY) * unit,
              unit + 0.25,
              unit + 0.25,
            );
        }),
      );
      return;
    }
    const rotate = ([x, y, z]: Point) => {
      const u = x * co + z * si,
        v = z * co - x * si;
      return [u, y * ct - v * st, y * st + v * ct] as Point;
    };
    if (stride === undefined) {
      const stage = height * 0.89;
      ctx.save();
      ctx.globalAlpha = light;
      const glow = ctx.createRadialGradient(cx, cy, 0, cx, cy, width * 0.55);
      glow.addColorStop(0, "#315e59");
      glow.addColorStop(1, "rgba(32,67,66,0)");
      ctx.fillStyle = glow;
      ctx.fillRect(0, 0, width, height);
      ctx.strokeStyle = "#3b5957";
      ctx.lineWidth = width / 520;
      ctx.beginPath();
      for (let i = -6; i <= 6; i++) {
        ctx.moveTo(cx + i * width * 0.035, stage - height * 0.045);
        ctx.lineTo(cx + i * width * 0.18, height);
      }
      for (const h of [0, 0.025, 0.07, 0.13]) {
        ctx.moveTo(0, stage + height * h);
        ctx.lineTo(width, stage + height * h);
      }
      ctx.stroke();
      ctx.fillStyle = "#102026";
      ctx.beginPath();
      ctx.ellipse(cx, stage, width * 0.24, height * 0.032, 0, 0, Math.PI * 2);
      ctx.fill();
      const burst = Math.sin(Math.PI * ease((t - 0.19) / 0.64));
      ctx.globalAlpha = burst;
      for (let i = 0; i < 22; i++) {
        const a = i * 2.39996 + t * (i % 2 ? 1 : -1),
          radius = width * (0.19 + 0.29 * grow);
        const size = ((1 + (i % 3)) * width) / 230;
        ctx.fillStyle = i % 3 ? "#c6e8a5" : "#f8c66c";
        ctx.fillRect(cx + Math.cos(a) * radius, cy + Math.sin(a) * radius * 0.85, size, size);
      }
      ctx.restore();
    }
    const polygons = [];
    for (const face of faces) {
      const normal = rotate(face.normal);
      if (normal[2] < 0.001) continue;
      const pts = face.corners.map(([x, y, z]) => {
        if (stride === undefined) return rotate([x, y, z * grow]);
        // The lower voxels swing in opposite phases around the two hips.
        const leg = ease((y - footY * 0.35) / Math.max(1, footY * 0.65));
        const step = Math.sin(stride) * Math.tanh(x) * leg;
        return rotate([x, y - Math.max(0, step) * 0.85 - Math.sin(stride) ** 2 * 0.28, z + step * 1.4]);
      });
      polygons.push({ face, pts, depth: pts.reduce((a, p) => a + p[2], 0) / 4 });
    }
    polygons.sort((a, b) => a.depth - b.depth);
    ctx.lineWidth = Math.max(0.4, width / 760);
    ctx.lineJoin = "round";
    for (const { face, pts } of polygons) {
      ctx.globalAlpha = face.ink ? 1 : grow;
      const [nx, ny, nz] = rotate(face.lightNormal);
      const direct = Math.max(0, -0.52 * nx - 0.68 * ny + 0.52 * nz);
      const shadow = mix(face.color, [28, 72, 78], (1 - direct) * 0.47);
      const target = mix(shadow, [247, 252, 208], Math.max(0, direct - 0.67) * 0.65);
      ctx.fillStyle = rgb(face.ink && grow < 0.99 ? mix(INK, target, grow) : target);
      ctx.strokeStyle = ctx.fillStyle;
      ctx.beginPath();
      pts.forEach(([x, y, z], i) => {
        const k = (unit * 48) / (48 - z);
        if (i === 0) ctx.moveTo(cx + x * k, cy + y * k);
        else ctx.lineTo(cx + x * k, cy + y * k);
      });
      ctx.closePath();
      ctx.fill();
      ctx.stroke();
    }
    ctx.globalAlpha = 1;
  };
}

/** Return a stop function so navigation, replay and reduced motion share cleanup. */
export function animateEmergence(
  canvas: HTMLCanvasElement,
  rows: string[],
  character: string,
  done: () => void,
) {
  const draw = emergenceRenderer(canvas, rows, character);
  const motion = window.matchMedia("(prefers-reduced-motion: reduce)");
  let frame = 0;
  let complete = false;
  let angle = 0;
  let held: { id: number; x: number; angle: number } | null = null;
  const originalLabel = canvas.getAttribute("aria-label");
  function turn(radians: number) {
    angle = radians;
    draw(emergenceDuration, angle);
    canvas.setAttribute(
      "aria-valuenow",
      String(((Math.round((angle * 180) / Math.PI) % 360) + 360) % 360),
    );
  }
  function press(event: PointerEvent) {
    if (!complete || event.button !== 0) return;
    held = { id: event.pointerId, x: event.clientX, angle };
    canvas.setPointerCapture(event.pointerId);
  }
  function drag(event: PointerEvent) {
    if (held?.id === event.pointerId)
      turn(held.angle + ((event.clientX - held.x) * Math.PI * 2) / Math.max(1, canvas.clientWidth));
  }
  function release(event: PointerEvent) {
    if (held?.id === event.pointerId) held = null;
  }
  function key(event: KeyboardEvent) {
    if (!complete || !["ArrowLeft", "ArrowRight", "Home", "End"].includes(event.key)) return;
    event.preventDefault();
    if (event.key === "Home" || event.key === "End")
      turn(event.key === "Home" ? 0 : Math.PI * 2 - Math.PI / 180);
    else turn(angle + ((event.key === "ArrowLeft" ? -1 : 1) * Math.PI) / 12);
  }
  const finish = () => {
    if (complete) return;
    complete = true;
    cancelAnimationFrame(frame);
    canvas.tabIndex = 0;
    canvas.setAttribute("role", "slider");
    canvas.setAttribute("aria-label", `Rotate ${character}`);
    canvas.setAttribute("aria-valuemin", "0");
    canvas.setAttribute("aria-valuemax", "359");
    turn(0);
    done();
  };
  const changed = () => {
    if (motion.matches) finish();
  };
  motion.addEventListener("change", changed);
  canvas.addEventListener("pointerdown", press);
  canvas.addEventListener("pointermove", drag);
  canvas.addEventListener("lostpointercapture", release);
  canvas.addEventListener("keydown", key);
  if (motion.matches) finish();
  else {
    let start: number | undefined;
    const tick = (now: number) => {
      start ??= now;
      const elapsed = now - start;
      if (elapsed >= emergenceDuration) {
        finish();
        return;
      }
      draw(elapsed);
      frame = requestAnimationFrame(tick);
    };
    frame = requestAnimationFrame(tick);
  }
  return () => {
    cancelAnimationFrame(frame);
    motion.removeEventListener("change", changed);
    canvas.removeEventListener("pointerdown", press);
    canvas.removeEventListener("pointermove", drag);
    canvas.removeEventListener("lostpointercapture", release);
    canvas.removeEventListener("keydown", key);
    for (const attribute of ["role", "tabindex", "aria-valuemin", "aria-valuemax", "aria-valuenow"])
      canvas.removeAttribute(attribute);
    if (originalLabel) canvas.setAttribute("aria-label", originalLabel);
  };
}
