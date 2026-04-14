/**
 * Agent result store — holds the latest RenderEnvelope from agent/query responses.
 * Components subscribe to this to render tables, map overlays, and panels.
 */

import { writable, derived } from 'svelte/store';

export interface AgentEnvelope {
  status: string;
  title: string;
  summary: string;
  columns: string[];
  data: Record<string, unknown>[];
  render_as: string[];
  geometry: GeoJSON.FeatureCollection | null;
  actions: unknown[];
  warnings: string[];
}

const EMPTY: AgentEnvelope = {
  status: '',
  title: '',
  summary: '',
  columns: [],
  data: [],
  render_as: [],
  geometry: null,
  actions: [],
  warnings: [],
};

export const agentResultStore = writable<AgentEnvelope>({ ...EMPTY });

export const hasAgentResult = derived(agentResultStore, ($r) => $r.status !== '');
export const agentShouldShowTable = derived(agentResultStore, ($r) =>
  $r.status === 'ok' && $r.render_as.includes('table') && $r.data.length > 0
);
export const agentShouldShowMap = derived(agentResultStore, ($r) =>
  $r.status === 'ok' && $r.render_as.includes('map') && $r.geometry !== null
);

export function setAgentResult(envelope: AgentEnvelope) {
  agentResultStore.set(envelope);
}

export function clearAgentResult() {
  agentResultStore.set({ ...EMPTY });
}
