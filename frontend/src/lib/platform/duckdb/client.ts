/**
 * DuckDB-WASM client — platform-owned (§7)
 * Initializes DuckDB in the browser, loads synthetic data,
 * and provides a query interface.
 */

import * as duckdb from '@duckdb/duckdb-wasm';
import { writable, get } from 'svelte/store';

export type DuckDBStatus = 'uninitialized' | 'loading' | 'ready' | 'error';

export const duckdbStatus = writable<DuckDBStatus>('uninitialized');
export const duckdbError = writable<string | null>(null);

let db: duckdb.AsyncDuckDB | null = null;
let conn: duckdb.AsyncDuckDBConnection | null = null;

/**
 * Initialize DuckDB-WASM and load synthetic vessel data into it.
 */
export async function initDuckDB(vessels: Array<{
  mmsi: number;
  name: string;
  lat: number;
  lon: number;
  speed: number;
  heading: number;
  vessel_type: string;
  timestamp: string;
}>): Promise<void> {
  if (get(duckdbStatus) === 'ready' || get(duckdbStatus) === 'loading') return;

  duckdbStatus.set('loading');

  try {
    // Select bundles — use MVP (smallest) for broad compatibility
    const DUCKDB_BUNDLES = await duckdb.selectBundle({
      mvp: {
        mainModule: new URL('@duckdb/duckdb-wasm/dist/duckdb-mvp.wasm', import.meta.url).href,
        mainWorker: new URL('@duckdb/duckdb-wasm/dist/duckdb-browser-mvp.worker.js', import.meta.url).href,
      },
      eh: {
        mainModule: new URL('@duckdb/duckdb-wasm/dist/duckdb-eh.wasm', import.meta.url).href,
        mainWorker: new URL('@duckdb/duckdb-wasm/dist/duckdb-browser-eh.worker.js', import.meta.url).href,
      },
    });

    const worker = new Worker(DUCKDB_BUNDLES.mainWorker!);
    const logger = new duckdb.ConsoleLogger(duckdb.LogLevel.WARNING);
    db = new duckdb.AsyncDuckDB(logger, worker);
    await db.instantiate(DUCKDB_BUNDLES.mainModule);

    conn = await db.connect();

    // Create ais_positions table and load data
    await conn.query(`
      CREATE TABLE ais_positions (
        mmsi INTEGER,
        name VARCHAR,
        lat DOUBLE,
        lon DOUBLE,
        speed DOUBLE,
        heading DOUBLE,
        vessel_type VARCHAR,
        timestamp TIMESTAMP
      )
    `);

    // Insert vessel data in batches
    const batchSize = 50;
    for (let i = 0; i < vessels.length; i += batchSize) {
      const batch = vessels.slice(i, i + batchSize);
      const values = batch
        .map(
          (v) =>
            `(${v.mmsi}, '${v.name.replace(/'/g, "''")}', ${v.lat}, ${v.lon}, ${v.speed}, ${v.heading}, '${v.vessel_type}', '${v.timestamp}')`
        )
        .join(',\n');
      await conn.query(`INSERT INTO ais_positions VALUES ${values}`);
    }

    // Create views from skill SQL definitions
    await conn.query(`
      CREATE OR REPLACE VIEW v_latest_positions AS
      SELECT DISTINCT ON (mmsi)
        mmsi, name, lat, lon, speed, heading, vessel_type, timestamp
      FROM ais_positions
      ORDER BY mmsi, timestamp DESC
    `);

    await conn.query(`
      CREATE OR REPLACE VIEW v_vessel_tracks AS
      SELECT mmsi, name, lat, lon, speed, heading, vessel_type, timestamp
      FROM ais_positions
      ORDER BY mmsi, timestamp ASC
    `);

    await conn.query(`
      CREATE OR REPLACE VIEW v_vessel_summary AS
      SELECT
        mmsi,
        name,
        vessel_type,
        COUNT(*) as position_count,
        AVG(speed) as avg_speed,
        MAX(speed) as max_speed,
        MIN(timestamp) as first_seen,
        MAX(timestamp) as last_seen
      FROM ais_positions
      GROUP BY mmsi, name, vessel_type
    `);

    console.log('[DuckDB] Initialized with', vessels.length, 'vessel positions');
    duckdbStatus.set('ready');
  } catch (err) {
    console.error('[DuckDB] Init failed:', err);
    duckdbError.set(String(err));
    duckdbStatus.set('error');
  }
}

/**
 * Execute a read-only SQL query against the browser DuckDB instance.
 * Returns results as an array of plain objects.
 */
export async function queryDuckDB<T = Record<string, unknown>>(
  sql: string
): Promise<T[]> {
  if (!conn) throw new Error('DuckDB not initialized');

  const result = await conn.query(sql);
  const rows: T[] = [];
  const schema = result.schema.fields;

  for (let i = 0; i < result.numRows; i++) {
    const row: Record<string, unknown> = {};
    for (const field of schema) {
      const col = result.getChildAt(schema.indexOf(field));
      row[field.name] = col?.get(i);
    }
    rows.push(row as T);
  }

  return rows;
}

/**
 * Query vessels within a bounding box
 */
export async function queryVesselsInBBox(
  minLon: number,
  minLat: number,
  maxLon: number,
  maxLat: number
): Promise<Record<string, unknown>[]> {
  return queryDuckDB(`
    SELECT mmsi, name, lat, lon, speed, heading, vessel_type, timestamp
    FROM v_latest_positions
    WHERE lon BETWEEN ${minLon} AND ${maxLon}
      AND lat BETWEEN ${minLat} AND ${maxLat}
    LIMIT 500
  `);
}

/**
 * Query vessels within a polygon using point-in-polygon via ray casting.
 * DuckDB-WASM doesn't have spatial extension in browser, so we do this
 * by querying all points in the bounding box of the polygon, then
 * filtering client-side with point-in-polygon.
 */
export async function queryVesselsInPolygon(
  polygon: number[][] // [[lon, lat], ...]
): Promise<Record<string, unknown>[]> {
  // Get bounding box of polygon
  let minLon = Infinity, minLat = Infinity, maxLon = -Infinity, maxLat = -Infinity;
  for (const [lon, lat] of polygon) {
    minLon = Math.min(minLon, lon);
    minLat = Math.min(minLat, lat);
    maxLon = Math.max(maxLon, lon);
    maxLat = Math.max(maxLat, lat);
  }

  const candidates = await queryVesselsInBBox(minLon, minLat, maxLon, maxLat);

  // Ray-casting point-in-polygon filter
  return candidates.filter((v) => {
    const x = Number(v.lon);
    const y = Number(v.lat);
    return pointInPolygon(x, y, polygon);
  });
}

/**
 * Query vessels within a radius (in km) of a center point.
 */
export async function queryVesselsInRadius(
  centerLon: number,
  centerLat: number,
  radiusKm: number
): Promise<Record<string, unknown>[]> {
  // Approximate bounding box
  const latDelta = radiusKm / 111.32;
  const lonDelta = radiusKm / (111.32 * Math.cos((centerLat * Math.PI) / 180));

  const candidates = await queryVesselsInBBox(
    centerLon - lonDelta,
    centerLat - latDelta,
    centerLon + lonDelta,
    centerLat + latDelta
  );

  // Filter by haversine distance
  return candidates.filter((v) => {
    const dist = haversineKm(centerLat, centerLon, Number(v.lat), Number(v.lon));
    return dist <= radiusKm;
  });
}

/** Run an ad-hoc query (validated: must be SELECT, bounded with LIMIT) */
export async function queryAdHoc(sql: string): Promise<Record<string, unknown>[]> {
  const trimmed = sql.trim().toUpperCase();
  if (!trimmed.startsWith('SELECT')) {
    throw new Error('Only SELECT queries are allowed');
  }
  // Enforce LIMIT if not present
  const bounded = trimmed.includes('LIMIT') ? sql : `${sql} LIMIT 1000`;
  return queryDuckDB(bounded);
}

// --- Utility functions ---

function pointInPolygon(x: number, y: number, polygon: number[][]): boolean {
  let inside = false;
  for (let i = 0, j = polygon.length - 1; i < polygon.length; j = i++) {
    const xi = polygon[i][0], yi = polygon[i][1];
    const xj = polygon[j][0], yj = polygon[j][1];
    const intersect = yi > y !== yj > y && x < ((xj - xi) * (y - yi)) / (yj - yi) + xi;
    if (intersect) inside = !inside;
  }
  return inside;
}

function haversineKm(lat1: number, lon1: number, lat2: number, lon2: number): number {
  const R = 6371;
  const dLat = ((lat2 - lat1) * Math.PI) / 180;
  const dLon = ((lon2 - lon1) * Math.PI) / 180;
  const a =
    Math.sin(dLat / 2) ** 2 +
    Math.cos((lat1 * Math.PI) / 180) *
      Math.cos((lat2 * Math.PI) / 180) *
      Math.sin(dLon / 2) ** 2;
  return R * 2 * Math.atan2(Math.sqrt(a), Math.sqrt(1 - a));
}
