<script lang="ts">
  /** Draggable divider. `dir="x"` resizes a width, `dir="y"` a height. `invert` flips the drag direction. */
  let {
    dir,
    value,
    min,
    max,
    invert = false,
    label,
    onchange,
    oncommit,
    onreset,
  }: {
    dir: "x" | "y";
    value: number;
    min: number;
    max: number;
    invert?: boolean;
    label: string;
    onchange: (v: number) => void;
    oncommit: () => void;
    onreset: () => void;
  } = $props();

  let start = 0;
  let startValue = 0;
  let active = $state(false);
  const clamp = (v: number) => Math.min(max, Math.max(min, Math.round(v)));

  function down(e: PointerEvent) {
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    start = dir === "x" ? e.clientX : e.clientY;
    startValue = value;
    active = true;
  }
  function move(e: PointerEvent) {
    if (!active) return;
    const d = (dir === "x" ? e.clientX : e.clientY) - start;
    onchange(clamp(startValue + (invert ? -d : d)));
  }
  function up() {
    if (!active) return;
    active = false;
    oncommit();
  }
  function key(e: KeyboardEvent) {
    const step = e.shiftKey ? 48 : 16;
    const grow = dir === "x" ? (invert ? "ArrowLeft" : "ArrowRight") : invert ? "ArrowUp" : "ArrowDown";
    const shrink = dir === "x" ? (invert ? "ArrowRight" : "ArrowLeft") : invert ? "ArrowDown" : "ArrowUp";
    if (e.key === grow) onchange(clamp(value + step));
    else if (e.key === shrink) onchange(clamp(value - step));
    else if (e.key === "Home") onchange(min);
    else if (e.key === "End") onchange(max);
    else return;
    e.preventDefault();
    oncommit();
  }
</script>

<!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
<div
  class="split {dir}"
  class:active
  role="separator"
  aria-orientation={dir === "x" ? "vertical" : "horizontal"}
  aria-label={label}
  aria-valuenow={Math.round(value)}
  aria-valuemin={min}
  aria-valuemax={max}
  tabindex="0"
  title="Drag to resize · double-click to reset"
  onpointerdown={down}
  onpointermove={move}
  onpointerup={up}
  onpointercancel={up}
  ondblclick={onreset}
  onkeydown={key}
></div>

<style>
  .split { position: relative; flex: none; background: transparent; z-index: 5; touch-action: none; }
  .split.x { width: 7px; margin: 0 -3px; cursor: col-resize; }
  .split.y { height: 7px; margin: -3px 0; cursor: row-resize; }
  .split::after { content: ""; position: absolute; background: transparent; transition: background 0.12s; }
  .split.x::after { left: 3px; top: 0; bottom: 0; width: 1px; }
  .split.y::after { top: 3px; left: 0; right: 0; height: 1px; }
  .split:hover::after, .split.active::after, .split:focus-visible::after { background: var(--accent); }
  .split.x:hover::after, .split.x.active::after, .split.x:focus-visible::after { left: 2px; width: 3px; }
  .split.y:hover::after, .split.y.active::after, .split.y:focus-visible::after { top: 2px; height: 3px; }
  .split:focus-visible { outline: none; }
</style>
