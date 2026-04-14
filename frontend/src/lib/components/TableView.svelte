<script lang="ts">
  import { selectedItems, selectionCount, hasSelection } from '$lib/platform/selection/store';
  import { selectVessel } from '$lib/platform/selection/store';
  import { mapStore } from '$lib/platform/map/store';
  import { agentResultStore, agentShouldShowTable, clearAgentResult } from '$lib/platform/render/store';
  import { get } from 'svelte/store';

  let expanded = $state(true);
  let sortKey = $state('');
  let sortAsc = $state(true);

  // Determine which data source is active: agent result or selection
  let source = $state<'agent' | 'selection' | null>(null);
  let displayTitle = $state('');
  let displayColumns: { key: string; label: string }[] = $state([]);
  let displayRows: Record<string, unknown>[] = $state([]);

  // Subscribe to agent results
  const unsubAgent = agentResultStore.subscribe(r => {
    if (r.status === 'ok' && r.render_as.includes('table') && r.data.length > 0) {
      source = 'agent';
      displayTitle = r.title || `${r.data.length} results`;
      // Auto-detect columns from data keys or use provided columns
      if (r.columns.length > 0) {
        displayColumns = r.columns.map(c => ({
          key: c,
          label: c.replace(/_/g, ' ').replace(/\b\w/g, ch => ch.toUpperCase()),
        }));
      } else if (r.data.length > 0) {
        displayColumns = Object.keys(r.data[0]).map(c => ({
          key: c,
          label: c.replace(/_/g, ' ').replace(/\b\w/g, ch => ch.toUpperCase()),
        }));
      }
      displayRows = r.data;
      expanded = true;
    } else if (source === 'agent') {
      // Agent result cleared — fall back to selection if available
      source = null;
    }
  });

  // Subscribe to selection as fallback
  const unsubSel = selectedItems.subscribe(items => {
    if (source !== 'agent' && items.length > 1) {
      source = 'selection';
      displayTitle = `${items.length} vessels selected`;
      displayColumns = [
        { key: 'name', label: 'Name' },
        { key: 'mmsi', label: 'MMSI' },
        { key: 'vessel_type', label: 'Type' },
        { key: 'speed', label: 'Speed' },
        { key: 'heading', label: 'Hdg' },
      ];
      displayRows = items as Record<string, unknown>[];
    } else if (source === 'selection') {
      source = null;
    }
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
    } else {
      sortKey = key;
      sortAsc = true;
    }
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
    if (value == null) return '\u2014';
    if (key === 'speed') return `${value} kn`;
    if (key === 'heading') return `${value}\u00B0`;
    if (key === 'lat' || key === 'lon') return Number(value).toFixed(4);
    if (typeof value === 'number' && !Number.isInteger(value)) return Number(value).toFixed(2);
    return String(value);
  }
</script>

{#if source !== null && displayRows.length > 0}
  <div class="table-view" class:expanded>
    <div class="table-header">
      <button class="toggle-btn" onclick={() => expanded = !expanded}>
        <span class="toggle-icon">{expanded ? '\u25BC' : '\u25B6'}</span>
        {displayTitle}
      </button>
      {#if source === 'agent'}
        <button class="close-btn" onclick={() => { clearAgentResult(); source = null; }}
          title="Close results">&times;</button>
      {/if}
    </div>

    {#if expanded}
      <div class="table-wrapper">
        <table>
          <thead>
            <tr>
              {#each displayColumns as col}
                <th
                  class:sorted={sortKey === col.key}
                  onclick={() => toggleSort(col.key)}
                >
                  {col.label}
                  {#if sortKey === col.key}
                    <span class="sort-arrow">{sortAsc ? '\u2191' : '\u2193'}</span>
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
    {/if}
  </div>
{/if}

<style>
  .table-view {
    position: absolute;
    bottom: 60px;
    left: 16px;
    z-index: 10;
    background: rgba(15, 20, 30, 0.92);
    backdrop-filter: blur(12px);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    color: #e0e0e0;
    font-family: 'SF Mono', 'Fira Code', monospace;
    font-size: 12px;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.5);
    max-width: min(640px, calc(100vw - 340px));
  }

  .table-view.expanded {
    max-height: 340px;
  }

  .table-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .toggle-btn {
    background: none;
    border: none;
    color: #FFD700;
    font-family: 'SF Mono', 'Fira Code', monospace;
    font-size: 11px;
    padding: 8px 12px;
    cursor: pointer;
    flex: 1;
    text-align: left;
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .toggle-btn:hover {
    background: rgba(255, 255, 255, 0.05);
  }

  .toggle-icon {
    font-size: 9px;
  }

  .close-btn {
    background: none;
    border: none;
    color: #666;
    font-size: 16px;
    padding: 4px 12px;
    cursor: pointer;
    line-height: 1;
  }

  .close-btn:hover {
    color: #ff6b6b;
  }

  .table-wrapper {
    max-height: 280px;
    overflow-y: auto;
    padding: 0 8px 8px;
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
    padding: 4px 8px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
    position: sticky;
    top: 0;
    background: rgba(15, 20, 30, 0.98);
    cursor: pointer;
    user-select: none;
    white-space: nowrap;
  }

  th:hover {
    color: #bbb;
  }

  th.sorted {
    color: #FFD700;
  }

  .sort-arrow {
    font-size: 9px;
    margin-left: 2px;
  }

  td {
    padding: 4px 8px;
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
</style>
