/**
 * Render envelope router — platform-owned (§4)
 * Takes a RenderEnvelope and dispatches to appropriate renderers.
 */

import type { RenderEnvelope, RenderTarget, ColumnDef } from '$lib/platform/types';

/**
 * Create a successful render envelope
 */
export function createEnvelope(opts: {
  title?: string;
  summary?: string;
  data?: Record<string, unknown> | unknown[];
  render_as?: RenderTarget[];
  columns?: ColumnDef[];
  geometry?: GeoJSON.FeatureCollection | null;
  actions?: { id: string; label: string; icon?: string }[];
  warnings?: string[];
}): RenderEnvelope {
  return {
    status: 'ok',
    title: opts.title ?? '',
    summary: opts.summary ?? '',
    data: opts.data ?? {},
    render_as: opts.render_as ?? [],
    columns: opts.columns ?? [],
    geometry: opts.geometry ?? null,
    actions: opts.actions ?? [],
    warnings: opts.warnings ?? [],
  };
}

/**
 * Create an error envelope
 */
export function createErrorEnvelope(message: string): RenderEnvelope {
  return {
    status: 'error',
    title: 'Error',
    summary: message,
    data: {},
    render_as: [],
    columns: [],
    geometry: null,
    actions: [],
    warnings: [message],
  };
}

/**
 * Convert an array of row objects to a render envelope with auto-detected columns
 */
export function rowsToEnvelope(
  rows: Record<string, unknown>[],
  opts?: { title?: string; summary?: string; includeGeometry?: boolean }
): RenderEnvelope {
  if (rows.length === 0) {
    return createEnvelope({
      title: opts?.title ?? 'Query Result',
      summary: 'No results found.',
      render_as: ['table'],
      columns: [],
      data: [],
    });
  }

  // Auto-detect columns from first row
  const columns: ColumnDef[] = Object.keys(rows[0]).map((key) => {
    const sample = rows[0][key];
    let type: ColumnDef['type'] = 'string';
    if (typeof sample === 'number') type = 'number';
    else if (typeof sample === 'boolean') type = 'boolean';
    else if (sample instanceof Date || (typeof sample === 'string' && !isNaN(Date.parse(sample)) && key.includes('time'))) {
      type = 'date';
    }

    return {
      key,
      label: key.replace(/_/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase()),
      type,
    };
  });

  const render_as: RenderTarget[] = ['table'];

  // If rows have lat/lon, also render on map
  let geometry: GeoJSON.FeatureCollection | null = null;
  if (opts?.includeGeometry !== false && rows[0].lat !== undefined && rows[0].lon !== undefined) {
    render_as.push('map');
    geometry = {
      type: 'FeatureCollection',
      features: rows.map((r) => ({
        type: 'Feature' as const,
        geometry: {
          type: 'Point' as const,
          coordinates: [Number(r.lon), Number(r.lat)],
        },
        properties: { ...r },
      })),
    };
  }

  // Single entity → also show panel
  if (rows.length === 1) {
    render_as.push('panel');
  }

  return createEnvelope({
    title: opts?.title ?? `${rows.length} result${rows.length !== 1 ? 's' : ''}`,
    summary: opts?.summary ?? `Found ${rows.length} record${rows.length !== 1 ? 's' : ''}.`,
    data: rows,
    render_as,
    columns,
    geometry,
  });
}
