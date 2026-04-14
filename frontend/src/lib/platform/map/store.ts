/**
 * MapLibre instance store — platform-owned
 */

import { writable } from 'svelte/store';
import type { Map as MapLibreMap } from 'maplibre-gl';

export const mapStore = writable<MapLibreMap | null>(null);
