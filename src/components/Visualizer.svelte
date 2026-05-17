<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { onDestroy, onMount } from "svelte";

  type Frame = { bars: number[] };
  type StateEvent = { state: "connecting" | "connected" | "disconnected" | "error"; error: string | null };

  let canvas = $state<HTMLCanvasElement | null>(null);
  let bars = $state<number[]>(new Array(32).fill(0));
  let conn = $state<StateEvent["state"]>("connecting");
  let unlistenFrame: UnlistenFn | null = null;
  let unlistenState: UnlistenFn | null = null;
  let rafId = 0;

  onMount(async () => {
    unlistenFrame = await listen<Frame>("viz:frame", (e) => {
      bars = e.payload.bars;
    });
    unlistenState = await listen<StateEvent>("viz:state", (e) => {
      conn = e.payload.state;
    });
    await invoke("visualizer_start");
    tick();
  });

  onDestroy(() => {
    unlistenFrame?.();
    unlistenState?.();
    cancelAnimationFrame(rafId);
    invoke("visualizer_stop").catch(() => {});
  });

  function tick() {
    draw();
    rafId = requestAnimationFrame(tick);
  }

  function draw() {
    const c = canvas;
    if (!c) return;
    const ctx = c.getContext("2d");
    if (!ctx) return;
    // Match backing store to CSS size + DPR so bars stay crisp on retina.
    const dpr = window.devicePixelRatio || 1;
    const w = c.clientWidth;
    const h = c.clientHeight;
    if (c.width !== Math.floor(w * dpr) || c.height !== Math.floor(h * dpr)) {
      c.width = Math.floor(w * dpr);
      c.height = Math.floor(h * dpr);
    }
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.clearRect(0, 0, w, h);

    const n = bars.length;
    if (!n) return;
    const gap = 3;
    const barW = (w - gap * (n - 1)) / n;
    // Gradient: accent at top, fading to translucent at bottom. Picks
    // up the theme accent variable so it matches the rest of the UI.
    const style = getComputedStyle(document.documentElement);
    const accent = style.getPropertyValue("--accent").trim() || "#7ad";
    const grad = ctx.createLinearGradient(0, 0, 0, h);
    grad.addColorStop(0, accent);
    grad.addColorStop(1, hexWithAlpha(accent, 0.25));
    ctx.fillStyle = grad;

    for (let i = 0; i < n; i++) {
      const v = Math.max(0, Math.min(1, bars[i]));
      const bh = v * h;
      const x = i * (barW + gap);
      const y = h - bh;
      // Slight rounded top edge — purely cosmetic, 2 px radius.
      const r = Math.min(2, barW / 2);
      ctx.beginPath();
      ctx.moveTo(x, y + r);
      ctx.quadraticCurveTo(x, y, x + r, y);
      ctx.lineTo(x + barW - r, y);
      ctx.quadraticCurveTo(x + barW, y, x + barW, y + r);
      ctx.lineTo(x + barW, h);
      ctx.lineTo(x, h);
      ctx.closePath();
      ctx.fill();
    }
  }

  function hexWithAlpha(hex: string, alpha: number): string {
    // Accept #rgb, #rrggbb, or already-rgb()/hsl() values. We only special-case
    // hex; for anything else just return it (the gradient end stop will be
    // less faded but still works).
    const m = /^#([0-9a-f]{3}|[0-9a-f]{6})$/i.exec(hex);
    if (!m) return hex;
    let r: number, g: number, b: number;
    if (m[1].length === 3) {
      r = parseInt(m[1][0] + m[1][0], 16);
      g = parseInt(m[1][1] + m[1][1], 16);
      b = parseInt(m[1][2] + m[1][2], 16);
    } else {
      r = parseInt(m[1].slice(0, 2), 16);
      g = parseInt(m[1].slice(2, 4), 16);
      b = parseInt(m[1].slice(4, 6), 16);
    }
    return `rgba(${r}, ${g}, ${b}, ${alpha})`;
  }
</script>

<aside class="viz-pane">
  <div class="qhead">
    <span class="qtitle">Visualizer</span>
    <span class="qcount" class:warn={conn !== "connected"}>{conn}</span>
  </div>
  <div class="canvas-wrap">
    <canvas bind:this={canvas}></canvas>
  </div>
</aside>

<style>
  .viz-pane {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-width: 0;
  }
  .qhead {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    padding: 12px 16px 8px;
  }
  .qtitle {
    font-size: 0.72rem;
    text-transform: uppercase;
    letter-spacing: 0.12em;
    color: var(--muted);
  }
  .qcount {
    font-size: 0.7rem;
    color: var(--muted);
  }
  .qcount.warn {
    color: var(--accent-warm, #c97);
  }
  .canvas-wrap {
    flex: 1;
    min-height: 0;
    padding: 8px 16px 16px;
  }
  canvas {
    width: 100%;
    height: 100%;
    display: block;
  }
</style>
