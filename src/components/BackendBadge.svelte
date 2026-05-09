<script lang="ts">
  type Props = { backend: string; size?: "sm" | "md" };
  let { backend, size = "sm" }: Props = $props();
</script>

<span class="badge {size}" data-backend={backend}>{backend.toUpperCase()}</span>

<style>
  .badge {
    display: inline-flex;
    align-items: center;
    padding: 2px 7px;
    border-radius: var(--radius-pill);
    font-size: 9.5px;
    font-weight: 700;
    letter-spacing: 0.06em;
    color: var(--badge-color, var(--text-mid));
    background: var(--badge-bg, var(--bg-2));
    border: 1px solid var(--badge-border, var(--border-soft));
    white-space: nowrap;
  }
  .badge.md {
    padding: 3px 9px;
    font-size: 10px;
  }
  .badge[data-backend="tidal"] {
    --badge-color: var(--backend-tidal);
    --badge-bg: rgba(51, 217, 217, 0.12);
    --badge-border: rgba(51, 217, 217, 0.4);
  }
  .badge[data-backend="local"] {
    --badge-color: var(--backend-local);
    --badge-bg: rgba(102, 217, 140, 0.12);
    --badge-border: rgba(102, 217, 140, 0.4);
  }
  .badge[data-backend="spotify"] {
    --badge-color: var(--backend-spotify);
    --badge-bg: rgba(102, 217, 102, 0.12);
    --badge-border: rgba(102, 217, 102, 0.4);
  }
  .badge[data-backend="podcast"] {
    --badge-color: var(--backend-podcast);
    --badge-bg: rgba(217, 140, 102, 0.12);
    --badge-border: rgba(217, 140, 102, 0.4);
  }
</style>
