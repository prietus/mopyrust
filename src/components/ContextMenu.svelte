<script lang="ts" module>
  export type MenuItem =
    | { kind: "item"; label: string; icon?: string; action: () => void; disabled?: boolean }
    | { kind: "separator" }
    | { kind: "submenu"; label: string; icon?: string; items: MenuItem[] };
</script>

<script lang="ts">
  import Icon from "./Icon.svelte";
  import Self from "./ContextMenu.svelte";

  type Props = {
    x: number;
    y: number;
    items: MenuItem[];
    onClose: () => void;
    /** Internal — submenus pass their parent's right edge to flip position. */
    parentRight?: number;
  };
  let { x, y, items, onClose, parentRight }: Props = $props();

  const MENU_W = 220;
  const ITEM_H = 30;
  const SEP_H = 9;

  let openSub = $state<{ idx: number; x: number; y: number } | null>(null);
  let menuEl: HTMLDivElement | undefined = $state();

  // Clamp menu within viewport — flip when overflowing.
  let pos = $derived.by(() => {
    const vw = window.innerWidth;
    const vh = window.innerHeight;
    const h = items.reduce(
      (a, i) => a + (i.kind === "separator" ? SEP_H : ITEM_H),
      8,
    );
    let nx = x;
    let ny = y;
    if (nx + MENU_W > vw - 8) {
      nx = parentRight !== undefined ? parentRight - MENU_W : vw - MENU_W - 8;
    }
    if (ny + h > vh - 8) ny = Math.max(8, vh - h - 8);
    return { x: nx, y: ny };
  });

  function activate(item: Extract<MenuItem, { kind: "item" }>) {
    if (item.disabled) return;
    item.action();
    onClose();
  }

  function onItemHover(idx: number, item: MenuItem, e: MouseEvent) {
    if (item.kind !== "submenu") {
      openSub = null;
      return;
    }
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    openSub = { idx, x: rect.right - 4, y: rect.top };
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === "Escape") {
      e.preventDefault();
      onClose();
    }
  }

  function onBackdrop(e: MouseEvent) {
    // Don't close when clicking inside any menu (parent or submenu).
    if (e.target instanceof Element && e.target.closest('.cm-root')) return;
    onClose();
  }

  $effect(() => {
    menuEl?.focus();
    window.addEventListener("mousedown", onBackdrop, true);
    window.addEventListener("scroll", onClose, true);
    window.addEventListener("resize", onClose);
    return () => {
      window.removeEventListener("mousedown", onBackdrop, true);
      window.removeEventListener("scroll", onClose, true);
      window.removeEventListener("resize", onClose);
    };
  });
</script>

<div
  bind:this={menuEl}
  class="cm cm-root"
  style="left: {pos.x}px; top: {pos.y}px; width: {MENU_W}px;"
  role="menu"
  tabindex="-1"
  onkeydown={onKey}
>
  {#each items as item, i (i)}
    {#if item.kind === "separator"}
      <div class="sep" role="separator"></div>
    {:else if item.kind === "submenu"}
      <button
        class="row sub"
        class:active={openSub?.idx === i}
        role="menuitem"
        onmouseenter={(e) => onItemHover(i, item, e)}
      >
        <span class="ic">{#if item.icon}<Icon name={item.icon} size={13} stroke={1.7} />{/if}</span>
        <span class="lbl">{item.label}</span>
        <span class="chev"><Icon name="chevron-right" size={12} stroke={1.7} /></span>
      </button>
    {:else}
      <button
        class="row"
        class:disabled={item.disabled}
        role="menuitem"
        disabled={item.disabled}
        onmouseenter={(e) => onItemHover(i, item, e)}
        onclick={() => activate(item)}
      >
        <span class="ic">{#if item.icon}<Icon name={item.icon} size={13} stroke={1.7} />{/if}</span>
        <span class="lbl">{item.label}</span>
      </button>
    {/if}
  {/each}
</div>

{#if openSub}
  {@const sub = items[openSub.idx]}
  {#if sub.kind === "submenu"}
    <Self
      x={openSub.x}
      y={openSub.y}
      items={sub.items}
      parentRight={pos.x}
      {onClose}
    />
  {/if}
{/if}

<style>
  .cm {
    position: fixed;
    z-index: 200;
    background: var(--bg-1);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-strong);
    padding: 4px;
    display: flex;
    flex-direction: column;
    outline: none;
    animation: pop 110ms ease-out;
  }
  @keyframes pop {
    from { opacity: 0; transform: translateY(-2px); }
    to { opacity: 1; transform: none; }
  }
  .row {
    display: grid;
    grid-template-columns: 18px 1fr auto;
    gap: 8px;
    align-items: center;
    padding: 6px 10px;
    background: transparent;
    border-radius: var(--radius-sm, 4px);
    color: var(--text);
    font-size: 12px;
    text-align: left;
    cursor: pointer;
    transition: background 80ms ease;
  }
  .row:hover:not(.disabled),
  .row.active {
    background: var(--bg-hover);
  }
  .row.disabled {
    color: var(--text-faint);
    cursor: default;
  }
  .ic {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    color: var(--text-muted);
  }
  .lbl {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .chev {
    color: var(--text-faint);
    display: inline-flex;
  }
  .sep {
    height: 1px;
    background: var(--border-soft);
    margin: 4px 6px;
  }
</style>
