<script lang="ts">
  import { drawModeStore, setDrawMode } from '$lib/platform/map/draw-mode';
  import { clearSelection } from '$lib/platform/selection/store';
  import { duckdbStatus } from '$lib/platform/duckdb/client';

  const tools = [
    { id: 'none', label: 'Select', icon: '&#9757;', title: 'Click to select vessels' },
    { id: 'polygon', label: 'Polygon', icon: '&#11039;', title: 'Draw polygon to select vessels' },
    { id: 'radius', label: 'Radius', icon: '&#9711;', title: 'Click to set center, drag for radius' },
  ] as const;
</script>

<div class="toolbar">
  {#each tools as tool}
    <button
      class="tool-btn"
      class:active={$drawModeStore === tool.id}
      title={tool.title}
      onclick={() => {
        setDrawMode(tool.id as 'none' | 'polygon' | 'radius');
        if (tool.id !== 'none') clearSelection();
      }}
    >
      <span class="tool-icon">{@html tool.icon}</span>
      <span class="tool-label">{tool.label}</span>
    </button>
  {/each}

  <div class="separator"></div>

  <div class="db-status" class:ready={$duckdbStatus === 'ready'} class:loading={$duckdbStatus === 'loading'} class:error={$duckdbStatus === 'error'}>
    {#if $duckdbStatus === 'loading'}
      DB...
    {:else if $duckdbStatus === 'ready'}
      DB
    {:else if $duckdbStatus === 'error'}
      DB!
    {:else}
      --
    {/if}
  </div>
</div>

<style>
  .toolbar {
    position: absolute;
    top: 10px;
    left: 10px;
    z-index: 10;
    display: flex;
    flex-direction: column;
    gap: 2px;
    background: rgba(15, 20, 30, 0.92);
    backdrop-filter: blur(12px);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    padding: 6px;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.5);
  }

  .tool-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    background: none;
    border: 1px solid transparent;
    color: #aaa;
    font-family: 'SF Mono', 'Fira Code', monospace;
    font-size: 11px;
    padding: 6px 10px;
    border-radius: 4px;
    cursor: pointer;
    white-space: nowrap;
  }

  .tool-btn:hover {
    background: rgba(255, 255, 255, 0.05);
    color: #e0e0e0;
  }

  .tool-btn.active {
    background: rgba(255, 215, 0, 0.12);
    color: #FFD700;
    border-color: rgba(255, 215, 0, 0.3);
  }

  .tool-icon {
    font-size: 14px;
    width: 18px;
    text-align: center;
  }

  .separator {
    height: 1px;
    background: rgba(255, 255, 255, 0.1);
    margin: 4px 0;
  }

  .db-status {
    font-family: 'SF Mono', 'Fira Code', monospace;
    font-size: 10px;
    padding: 4px 10px;
    text-align: center;
    color: #555;
    border-radius: 4px;
  }

  .db-status.ready {
    color: #27AE60;
  }

  .db-status.loading {
    color: #F39C12;
    animation: pulse 1s infinite;
  }

  .db-status.error {
    color: #E74C3C;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.4; }
  }
</style>
