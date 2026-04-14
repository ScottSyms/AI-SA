/**
 * Selection state store — platform-owned (§10, §11)
 */

import { writable, derived } from 'svelte/store';
import type { SelectionState, SelectionItem } from '$lib/platform/types';

const INITIAL_STATE: SelectionState = {
  mode: 'single',
  items: [],
  geometry: null,
  primaryMMSI: null,
};

export const selectionStore = writable<SelectionState>({ ...INITIAL_STATE });

export const selectedItems = derived(selectionStore, ($s) => $s.items);
export const primarySelection = derived(selectionStore, ($s) =>
  $s.items.find((i) => i.mmsi === $s.primaryMMSI) ?? $s.items[0] ?? null
);
export const hasSelection = derived(selectionStore, ($s) => $s.items.length > 0);
export const selectionCount = derived(selectionStore, ($s) => $s.items.length);

export function selectVessel(item: SelectionItem, multi: boolean = false) {
  selectionStore.update((state) => {
    if (multi) {
      const exists = state.items.find((i) => i.mmsi === item.mmsi);
      if (exists) {
        const items = state.items.filter((i) => i.mmsi !== item.mmsi);
        return {
          ...state,
          mode: 'multi' as const,
          items,
          primaryMMSI: items[0]?.mmsi ?? null,
        };
      }
      return {
        ...state,
        mode: 'multi' as const,
        items: [...state.items, item],
        primaryMMSI: state.primaryMMSI ?? item.mmsi,
      };
    }
    return {
      ...state,
      mode: 'single' as const,
      items: [item],
      primaryMMSI: item.mmsi,
    };
  });
}

export function clearSelection() {
  selectionStore.set({ ...INITIAL_STATE });
}

export function selectByGeometry(items: SelectionItem[], geometry: GeoJSON.Geometry) {
  selectionStore.set({
    mode: 'multi',
    items,
    geometry,
    primaryMMSI: items[0]?.mmsi ?? null,
  });
}
