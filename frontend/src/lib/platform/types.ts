/**
 * Platform-wide type definitions
 * These types form the rendering contract described in AGENTS.md §4
 */

// --- Render Envelope (§4) ---

export type RenderTarget = 'map' | 'table' | 'panel';

export interface RenderEnvelope {
  status: 'ok' | 'error';
  title: string;
  summary: string;
  data: Record<string, unknown> | unknown[];
  render_as: RenderTarget[];
  columns: ColumnDef[];
  geometry: GeoJSON.FeatureCollection | null;
  actions: Action[];
  warnings: string[];
}

export interface ColumnDef {
  key: string;
  label: string;
  type: 'string' | 'number' | 'date' | 'boolean';
}

export interface Action {
  id: string;
  label: string;
  icon?: string;
}

// --- Selection Model (§10) ---

export type SelectionMode = 'single' | 'multi';

export interface SelectionItem {
  mmsi: number;
  name: string;
  [key: string]: unknown;
}

export interface SelectionState {
  mode: SelectionMode;
  items: SelectionItem[];
  geometry: GeoJSON.Geometry | null;
  primaryMMSI: number | null;
}

// --- Skill System (§5) ---

export interface SkillMapLayer {
  id: string;
  type: 'circle' | 'line' | 'fill' | 'symbol';
}

export interface SkillManifest {
  skill: string;
  source_type: string;
  source_path: string;
  interactions: string[];
  map_layers: SkillMapLayer[];
  tools: ToolDef[];
  domain_prompt: string;
  sql_views: string[];
}

export interface ToolDef {
  name: string;
  description: string;
  parameters: Record<string, unknown>;
  sql?: string;
}

// --- Vessel (AIS domain — used by reference skill) ---

export interface Vessel {
  mmsi: number;
  name: string;
  lat: number;
  lon: number;
  speed: number;
  heading: number;
  vessel_type: string;
  timestamp: string;
}

export interface TrackPoint {
  mmsi: number;
  lat: number;
  lon: number;
  speed: number;
  heading: number;
  timestamp: string;
}
