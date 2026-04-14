/**
 * Drawing mode store — tracks which spatial tool is active
 */

import { writable } from 'svelte/store';

export type DrawMode = 'none' | 'polygon' | 'radius';

export const drawModeStore = writable<DrawMode>('none');

export function setDrawMode(mode: DrawMode) {
  drawModeStore.set(mode);
}
