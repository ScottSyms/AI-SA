/**
 * Backend API client.
 * Communicates with the Rust Axum server via the Vite dev proxy (/api/*).
 * Falls back gracefully if the backend is not running.
 */

import type { RenderEnvelope } from '../types';

const API_BASE = '/api';

export interface BackendHealth {
  status: string;
  service: string;
}

export interface BackendSkill {
  name: string;
  source_type: string;
  interactions: string[];
  map_layers: Array<{ id: string; type: string }>;
  tools: Array<{
    name: string;
    description: string;
    parameters: Array<{ name: string; param_type: string; required: boolean; default: string | null }>;
    sql_file: string | null;
  }>;
  domain_prompt: string;
}

export interface QueryResult {
  columns: string[];
  rows: Record<string, unknown>[];
  row_count: number;
}

/** Check if the backend is reachable. */
export async function checkHealth(): Promise<BackendHealth | null> {
  try {
    const res = await fetch(`${API_BASE}/health`);
    if (!res.ok) return null;
    return await res.json();
  } catch {
    return null;
  }
}

/** Fetch loaded skills from the backend. */
export async function fetchSkills(): Promise<BackendSkill[]> {
  const res = await fetch(`${API_BASE}/skills`);
  if (!res.ok) throw new Error(`skills fetch failed: ${res.status}`);
  const data = await res.json();
  return data.skills ?? [];
}

/** Execute a raw SQL query (must include LIMIT, read-only). */
export async function querySQL(sql: string): Promise<QueryResult> {
  const res = await fetch(`${API_BASE}/query`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ sql })
  });
  const data = await res.json();
  if (data.status !== 'ok') throw new Error(data.error ?? 'query failed');
  return data.data;
}

/** Execute a skill tool with parameters. */
export async function queryTool(
  skill: string,
  tool: string,
  params: Record<string, string | number> = {}
): Promise<QueryResult> {
  const res = await fetch(`${API_BASE}/query`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ skill, tool, params })
  });
  const data = await res.json();
  if (data.status !== 'ok') throw new Error(data.error ?? 'tool query failed');
  return data.data;
}

/** Send a message to the agent endpoint. Returns the full RenderEnvelope. */
export async function sendAgentMessage(
  message: string,
  context?: Record<string, unknown>
): Promise<AgentRenderEnvelope> {
  const res = await fetch(`${API_BASE}/agent`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ message, context })
  });
  const data = await res.json();
  if (data.status && data.status !== 'ok') throw new Error(data.summary ?? 'agent call failed');
  return {
    status: data.status ?? 'ok',
    title: data.title ?? '',
    summary: data.summary ?? '',
    columns: data.columns ?? [],
    data: Array.isArray(data.data) ? data.data : [],
    render_as: data.render_as ?? ['panel'],
    geometry: data.geometry ?? null,
    actions: data.actions ?? [],
    warnings: data.warnings ?? [],
  };
}

/** The shape returned by the agent endpoint (RenderEnvelope from backend). */
export interface AgentRenderEnvelope {
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
