import { useState, useEffect, useRef } from "react";

const w = 400;
const strokeWidth = 3 / 400;

function animate(fn: () => boolean) {
  function loop() {
    if (fn()) {
      requestAnimationFrame(loop);
    }
  }
  loop();
}

function d(hist: { x: number; y: number }[]) {
  let s = `M ${hist[0].x},${hist[0].y} L`;
  for (const { x, y } of hist) {
    s += ` ${x},${y}`;
  }
  return s;
}

const initialCursor = { x: 0, y: 0 };
const initialHist = [
  { x: 0, y: 0 },
  { x: 0, y: 0 },
  { x: 0, y: 0 },
  { x: 0, y: 0 },
  { x: 0, y: 0 },
  { x: 0, y: 0 },
  { x: 0, y: 0 },
  { x: 0, y: 0 },
  { x: 0, y: 0 },
  { x: 0, y: 0 },
  { x: 0, y: 0 },
  { x: 0, y: 0 },
];

const initialState = {
  cursor: initialCursor,
  hist: initialHist,
};

function Dot() {
  const [, setRender] = useState(false);
  const state = useRef(initialState);

  useEffect(() => {
    let alive = true;
    window.addEventListener("gamepadconnected", (ev) => {
      alive = true;
      let gamepad = navigator.getGamepads()[ev.gamepad.index]!;
      animate(() => {
        gamepad = navigator.getGamepads()[ev.gamepad.index]!;
        const [x, y] = gamepad.axes;
        if (state.current.cursor.x !== x || state.current.cursor.y !== y) {
          state.current.cursor.x = x;
          state.current.cursor.y = y;
          const [del] = state.current.hist.splice(0, 1);
          del.x = x;
          del.y = y;
          state.current.hist.push(del);
          setRender((r) => !r);
        }
        return alive;
      });
    });

    window.addEventListener("gamepaddisconnected", () => {
      alive = false;
    });

    return () => {
      alive = false;
    };
  }, []);

  return (
    <>
      <circle
        cx={state.current.cursor.x}
        cy={state.current.cursor.y}
        r={8 / w}
        fill="red"
      />
      <path
        fill="none"
        stroke="rgba(0,0,0,0.5)"
        strokeWidth={strokeWidth}
        d={d(state.current.hist)}
      />
      <path
        fill="none"
        stroke="white"
        strokeWidth={strokeWidth * 2}
        d={`M 0,0 L ${state.current.cursor.x},${state.current.cursor.y}`}
      />
    </>
  );
}

function App() {
  return (
    <div style={{ width: w + "px", height: w + "px" }}>
      <svg viewBox={`-2 -2 4 4`}>
        <rect x={-1} y={-1} width={2} height={1} fill="rgba(255,0,255,0.25)" />
        <rect x={0} y={-1} width={1} height={1} fill="rgba(255,0,0,0.25)" />
        <rect x={-1} y={0} width={1} height={1} fill="rgba(0,255,255,0.25)" />
        <rect x={0} y={0} width={1} height={1} fill="rgba(0,0,255,0.25)" />
        <circle
          cx={0}
          cy={0}
          r={1}
          stroke="black"
          strokeWidth={strokeWidth}
          fill="none"
        />
        <Dot />
      </svg>
    </div>
  );
}

export default App;
