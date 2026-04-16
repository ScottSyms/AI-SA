<script lang="ts">
  import { onDestroy } from 'svelte';
  import { get } from 'svelte/store';
  import { primarySelection, selectedItems, selectVessel } from '$lib/platform/selection/store';
  import { mapStore } from '$lib/platform/map/store';
  import { agentResultStore, clearAgentResult } from '$lib/platform/render/store';

  let sortKey = $state('');
  let sortAsc = $state(true);
  let source = $state<'agent' | 'selection' | 'detail' | null>(null);
  let displayTitle = $state('Results');
  let displayColumns: { key: string; label: string }[] = $state([]);
  let displayRows: Record<string, unknown>[] = $state([]);
  let detailItem = $state<Record<string, unknown> | null>(null);

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

  const unsubAgent = agentResultStore.subscribe((result) => {
    if (result.status === 'ok' && result.render_as.includes('table') && result.data.length > 0) {
      source = 'agent';
      displayTitle = result.title || `${result.data.length} results`;
      if (result.columns.length > 0) {
        displayColumns = result.columns.map((column) => ({
          key: column,
          label: column.replace(/_/g, ' ').replace(/\b\w/g, (ch) => ch.toUpperCase()),
        }));
      } else {
        displayColumns = Object.keys(result.data[0] ?? {}).map((column) => ({
          key: column,
          label: column.replace(/_/g, ' ').replace(/\b\w/g, (ch) => ch.toUpperCase()),
        }));
      }
      displayRows = result.data;
      return;
    }

    if (source === 'agent') {
      source = null;
      displayRows = [];
      displayColumns = [];
      displayTitle = 'Results';
    }
  });

  const unsubSelection = selectedItems.subscribe((items) => {
    if (source !== 'agent' && items.length > 1) {
      source = 'selection';
      displayTitle = `${items.length} vessels selected`;
      displayColumns = [
        { key: 'name', label: 'Name' },
        { key: 'mmsi', label: 'MMSI' },
        { key: 'vessel_type', label: 'Type' },
        { key: 'speed', label: 'Speed' },
        { key: 'heading', label: 'Hdg' },
        { key: 'lat', label: 'Lat' },
        { key: 'lon', label: 'Lon' },
      ];
      displayRows = items as Record<string, unknown>[];
      detailItem = null;
      return;
    }

    if (source !== 'agent' && items.length === 1) {
      source = 'detail';
      displayTitle = 'Vessel Detail';
      displayColumns = [];
      displayRows = [];
      detailItem = items[0] as Record<string, unknown>;
      return;
    }

    if (source === 'selection' || source === 'detail') {
      source = null;
      displayRows = [];
      displayColumns = [];
      displayTitle = 'Results';
      detailItem = null;
    }
  });

  const unsubPrimary = primarySelection.subscribe((item) => {
    if (source === 'detail') {
      detailItem = (item as Record<string, unknown> | null) ?? null;
    }
  });

  onDestroy(() => {
    unsubAgent();
    unsubSelection();
    unsubPrimary();
  });

  function sortedItems(items: Record<string, unknown>[]) {
    if (!sortKey) return items;
    return [...items].sort((a, b) => {
      const va = a[sortKey];
      const vb = b[sortKey];
      if (va == null && vb == null) return 0;
      if (va == null) return 1;
      if (vb == null) return -1;
      const cmp = typeof va === 'number' && typeof vb === 'number'
        ? va - vb
        : String(va).localeCompare(String(vb));
      return sortAsc ? cmp : -cmp;
    });
  }

  function toggleSort(key: string) {
    if (sortKey === key) {
      sortAsc = !sortAsc;
      return;
    }
    sortKey = key;
    sortAsc = true;
  }

  function handleRowClick(item: Record<string, unknown>) {
    const map = get(mapStore);
    if (map && item.lon != null && item.lat != null) {
      map.flyTo({
        center: [Number(item.lon), Number(item.lat)],
        zoom: Math.max(map.getZoom(), 8),
        duration: 800,
      });
    }
    if (item.mmsi != null && item.name != null) {
      selectVessel({ mmsi: Number(item.mmsi), name: String(item.name), ...item }, false);
    }
  }

  function formatCell(value: unknown, key: string): string {
    if (value == null) return '--';
    if (key === 'speed') return `${value} kn`;
    if (key === 'heading') return `${value} deg`;
    if (key === 'lat' || key === 'lon') return Number(value).toFixed(4);
    if (typeof value === 'number' && !Number.isInteger(value)) return Number(value).toFixed(2);
    return String(value);
  }

  function formatCoord(value: unknown): string {
    if (value == null || value === '') return '--';
    const n = Number(value);
    return Number.isFinite(n) ? n.toFixed(4) : '--';
  }

  function formatTime(ts: unknown): string {
    if (!ts) return '--';
    try {
      return new Date(String(ts)).toLocaleTimeString();
    } catch {
      return String(ts);
    }
  }
</script>

<div class="table-view">
  <div class="table-header">
    <div>
      <div class="table-title">{displayTitle}</div>
      <div class="table-subtitle">
        {#if displayRows.length > 0}
          {displayRows.length} row(s)
        {:else if source === 'detail' && detailItem}
          Single vessel selection
        {:else}
          Agent results and vessel details appear here.
        {/if}
      </div>
    </div>
    {#if source === 'agent'}
      <button class="close-btn" onclick={() => clearAgentResult()} title="Close results">Close</button>
    {/if}
  </div>

  {#if source === 'detail' && detailItem}
    <div class="detail-view">
      <div class="detail-grid">
        <div class="field">
          <span class="label">Name</span>
          <span class="value strong">{String(detailItem.name ?? '--')}</span>
        </div>

        <div class="field">
          <span class="label">MMSI</span>
          <span class="value mono">{detailItem.mmsi ?? '--'}</span>
        </div>

        <div class="field">
          <span class="label">Type</span>
          <span class="value">
            <span
              class="type-dot"
              style="background: {VESSEL_TYPE_COLORS[String(detailItem.vessel_type)] ?? '#888'}"
            ></span>
            {detailItem.vessel_type ?? '--'}
          </span>
        </div>

        <div class="field">
          <span class="label">Speed</span>
          <span class="value">{detailItem.speed ?? '--'} kn</span>
        </div>

        <div class="field">
          <span class="label">Heading</span>
          <span class="value">{detailItem.heading ?? '--'}°</span>
        </div>

        <div class="field">
          <span class="label">Position</span>
          <span class="value mono">{formatCoord(detailItem.lat)}, {formatCoord(detailItem.lon)}</span>
        </div>

        <div class="field">
          <span class="label">Last Update</span>
          <span class="value">{formatTime(detailItem.timestamp)}</span>
        </div>
      </div>
    </div>
  {:else if displayRows.length > 0}
    <div class="table-wrapper">
      <table>
        <thead>
          <tr>
            {#each displayColumns as col}
              <th class:sorted={sortKey === col.key} onclick={() => toggleSort(col.key)}>
                {col.label}
                {#if sortKey === col.key}
                  <span class="sort-arrow">{sortAsc ? '↑' : '↓'}</span>
                {/if}
              </th>
            {/each}
          </tr>
        </thead>
        <tbody>
          {#each sortedItems(displayRows) as item}
            <tr onclick={() => handleRowClick(item)} class="clickable">
              {#each displayColumns as col}
                <td class:mono={col.key === 'mmsi' || col.key === 'speed' || col.key === 'heading' || col.key === 'lat' || col.key === 'lon'}>
                  {formatCell(item[col.key], col.key)}
                </td>
              {/each}
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {:else}
    <div class="empty-state">
      Run a query or select multiple ships to populate the lower table pane.
    </div>
  {/if}
</div>

<style>
  .table-view {
    position: relative;
    height: 100%;
    min-height: 0;
    display: flex;
    flex-direction: column;
    background: rgba(15, 20, 30, 0.92);
    border-top: 1px solid rgba(255, 255, 255, 0.08);
    color: #e0e0e0;
    font-family: 'SF Mono', 'Fira Code', monospace;
    font-size: 12px;
  }

  .table-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 12px 16px 10px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
    flex-shrink: 0;
  }

  .table-title {
    color: #ffd700;
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }

  .table-subtitle {
    color: #6b7280;
    font-size: 11px;
    margin-top: 4px;
  }

  .close-btn {
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 8px;
    color: #d6dde8;
    cursor: pointer;
    font-family: inherit;
    font-size: 11px;
    padding: 8px 10px;
  }

  .detail-view {
    flex: 1;
    min-height: 0;
    overflow: auto;
    padding: 16px;
  }

  .detail-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 12px 20px;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 12px 14px;
    border-radius: 10px;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.06);
  }

  .label {
    color: #888;
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }

  .value {
    color: #e0e0e0;
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 14px;
  }

  .value.strong {
    font-size: 18px;
    font-weight: 700;
  }

  .mono {
    font-variant-numeric: tabular-nums;
  }

  .type-dot {
    display: inline-block;
    width: 10px;
    height: 10px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .table-wrapper {
    flex: 1;
    min-height: 0;
    overflow: auto;
    padding: 0 12px 76px;
  }

  table {
    width: 100%;
    border-collapse: collapse;
  }

  th {
    text-align: left;
    color: #888;
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    padding: 8px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
    position: sticky;
    top: 0;
    background: rgba(15, 20, 30, 0.98);
    cursor: pointer;
    user-select: none;
    white-space: nowrap;
  }

  th.sorted {
    color: #ffd700;
  }

  .sort-arrow {
    font-size: 9px;
    margin-left: 2px;
  }

  td {
    padding: 8px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.03);
    white-space: nowrap;
  }

  td.mono {
    font-variant-numeric: tabular-nums;
  }

  tr.clickable {
    cursor: pointer;
  }

  tr.clickable:hover td {
    background: rgba(255, 215, 0, 0.08);
  }

  .empty-state {
    flex: 1;
    min-height: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    text-align: center;
    color: #6b7280;
    padding: 24px;
  }

  @media (max-width: 900px) {
    .detail-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
