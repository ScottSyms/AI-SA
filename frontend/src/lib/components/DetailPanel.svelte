<script lang="ts">
  import { primarySelection, hasSelection, selectionCount, selectedItems } from '$lib/platform/selection/store';
  import { clearSelection } from '$lib/platform/selection/store';

  const VESSEL_TYPE_COLORS: Record<string, string> = {
    Cargo: '#4A90D9',
    Tanker: '#E74C3C',
    Fishing: '#27AE60',
    Passenger: '#F39C12',
    Tug: '#8E44AD',
    Sailing: '#1ABC9C',
    Pleasure: '#F1C40F',
    Military: '#2C3E50',
    Research: '#3498DB',
    Pilot: '#E67E22',
  };

  function formatCoord(val: unknown): string {
    const n = Number(val);
    return isNaN(n) ? String(val) : n.toFixed(4);
  }

  function formatTime(ts: unknown): string {
    if (!ts) return '—';
    try {
      return new Date(String(ts)).toLocaleTimeString();
    } catch {
      return String(ts);
    }
  }
</script>

{#if $hasSelection && $primarySelection}
  {@const vessel = $primarySelection}
  <div class="detail-panel">
    <div class="panel-header">
      <h2>{vessel.name}</h2>
      <button class="close-btn" onclick={() => clearSelection()}>✕</button>
    </div>

    {#if $selectionCount > 1}
      <div class="multi-badge">{$selectionCount} vessels selected</div>
    {/if}

    <div class="field">
      <span class="label">MMSI</span>
      <span class="value">{vessel.mmsi}</span>
    </div>

    <div class="field">
      <span class="label">Type</span>
      <span class="value">
        <span
          class="type-dot"
          style="background: {VESSEL_TYPE_COLORS[String(vessel.vessel_type)] ?? '#888'}"
        ></span>
        {vessel.vessel_type}
      </span>
    </div>

    <div class="field">
      <span class="label">Speed</span>
      <span class="value">{vessel.speed} kn</span>
    </div>

    <div class="field">
      <span class="label">Heading</span>
      <span class="value">{vessel.heading}°</span>
    </div>

    <div class="field">
      <span class="label">Position</span>
      <span class="value">{formatCoord(vessel.lat)}, {formatCoord(vessel.lon)}</span>
    </div>

    <div class="field">
      <span class="label">Last Update</span>
      <span class="value">{formatTime(vessel.timestamp)}</span>
    </div>

    {#if $selectionCount > 1}
      <div class="multi-list">
        <h3>Selected Vessels</h3>
        {#each $selectedItems as item}
          <div class="multi-item" class:primary={item.mmsi === vessel.mmsi}>
            <span class="type-dot" style="background: {VESSEL_TYPE_COLORS[String(item.vessel_type)] ?? '#888'}"></span>
            {item.name}
          </div>
        {/each}
      </div>
    {/if}
  </div>
{/if}

<style>
  .detail-panel {
    position: absolute;
    top: 10px;
    right: 50px;
    width: 300px;
    max-height: calc(100vh - 80px);
    overflow-y: auto;
    background: rgba(15, 20, 30, 0.92);
    backdrop-filter: blur(12px);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    padding: 16px;
    color: #e0e0e0;
    font-family: 'SF Mono', 'Fira Code', monospace;
    font-size: 13px;
    z-index: 10;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.5);
  }

  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
    padding-bottom: 8px;
  }

  .panel-header h2 {
    margin: 0;
    font-size: 16px;
    font-weight: 600;
    color: #ffffff;
  }

  .close-btn {
    background: none;
    border: none;
    color: #888;
    font-size: 16px;
    cursor: pointer;
    padding: 2px 6px;
    border-radius: 4px;
  }
  .close-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    color: #fff;
  }

  .multi-badge {
    background: rgba(255, 215, 0, 0.15);
    color: #FFD700;
    padding: 4px 8px;
    border-radius: 4px;
    font-size: 11px;
    margin-bottom: 10px;
    text-align: center;
  }

  .field {
    display: flex;
    justify-content: space-between;
    padding: 6px 0;
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
  }

  .label {
    color: #888;
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .value {
    color: #e0e0e0;
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .type-dot {
    display: inline-block;
    width: 8px;
    height: 8px;
    border-radius: 50%;
  }

  .multi-list {
    margin-top: 12px;
    border-top: 1px solid rgba(255, 255, 255, 0.1);
    padding-top: 8px;
  }

  .multi-list h3 {
    font-size: 11px;
    color: #888;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin: 0 0 8px 0;
  }

  .multi-item {
    padding: 4px 8px;
    border-radius: 4px;
    font-size: 12px;
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .multi-item.primary {
    background: rgba(255, 215, 0, 0.1);
  }
</style>
