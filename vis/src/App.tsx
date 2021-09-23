import { useState, useEffect } from "react";

const w = 250;
const w2 = 250 / 2;

function Dot() {
  const [state, setState] = useState({ x: 0, y: 0 });

  useEffect(() => {
    let alive = true;
    window.addEventListener("gamepadconnected", (ev) => {
      console.log("Got gamepad", ev.gamepad);
      const gamepad = ev.gamepad;
      function loop() {
        const [x, y] = gamepad.axes;
        console.log(gamepad.axes);
        setState({ x: x * w2 + w2, y: y * w2 + w2 });
        if (alive) requestAnimationFrame(loop);
      }
      loop();
    });
    return () => {
      alive = false;
    };
  }, []);

  return <circle cx={state.x} cy={state.y} r={4} fill="red" />;
}

function App() {
  return (
    <div style={{ width: w + "px", height: w + "px" }}>
      <svg viewBox={`-4 -4 ${w + 8} ${w + 8}`}>
        <rect
          x={0}
          y={0}
          width={w2}
          height={w2}
          fill="rgba(0,0,0,0.25)"
          stroke="black"
          strokeWidth={1}
        />
        <rect
          x={w2}
          y={0}
          width={w2}
          height={w2}
          fill="rgba(0,0,0,0.25)"
          stroke="black"
          strokeWidth={1}
        />
        <rect
          x={0}
          y={w2}
          width={w2}
          height={w2}
          fill="rgba(0,0,0,0.25)"
          stroke="black"
          strokeWidth={1}
        />
        <rect
          x={w2}
          y={w2}
          width={w2}
          height={w2}
          fill="rgba(0,0,0,0.25)"
          stroke="black"
          strokeWidth={1}
        />
        <circle
          cx={w2}
          cy={w2}
          r={w2}
          fill="rgba(0,0,0,0.5)"
          stroke="black"
          strokeWidth={1}
        />
        <Dot />
      </svg>
    </div>
  );
}

export default App;
