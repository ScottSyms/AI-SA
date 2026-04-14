/**
 * Synthetic AIS data generator
 * Produces ~200 ship positions and track histories for Phase 1
 */

import type { Vessel, TrackPoint } from '$lib/platform/types';

const VESSEL_NAMES = [
  'Atlantic Pioneer', 'Pacific Voyager', 'Northern Star', 'Southern Cross',
  'Sea Wanderer', 'Ocean Titan', 'Coral Venture', 'Arctic Explorer',
  'Storm Chaser', 'Blue Horizon', 'Golden Eagle', 'Silver Wave',
  'Red Falcon', 'Iron Maiden', 'Crystal Bay', 'Thunder Bay',
  'Wind Spirit', 'Polar Bear', 'Sun Dancer', 'Moon Shadow',
  "Neptune's Pride", 'Emerald Isle', 'Diamond Star', 'Ruby Queen',
  'Sapphire Dream', 'Pearl Harbor', 'Jade Emperor', 'Amber Dawn',
  'Crimson Tide', 'Azure Sky', 'Ivory Coast', 'Ebony Night',
  'Topaz Sun', 'Opal Moon', 'Garnet Fire', 'Onyx Shadow',
  'Turquoise Wave', 'Coral Reef', 'Lotus Flower', 'Dragon Wing',
  'Phoenix Rise', 'Falcon Crest', 'Eagle Eye', 'Hawk Wind',
  'Osprey Flight', 'Heron Bay', 'Pelican Point', 'Albatross Wing',
  'Marlin Strike', 'Swordfish Run'
];

const VESSEL_TYPES = [
  'Cargo', 'Tanker', 'Fishing', 'Passenger', 'Tug',
  'Sailing', 'Pleasure', 'Military', 'Research', 'Pilot'
];

function seededRandom(seed: number) {
  let s = seed;
  return () => {
    s = (s * 16807 + 0) % 2147483647;
    return (s - 1) / 2147483646;
  };
}

const rand = seededRandom(42);

function randomInRange(min: number, max: number): number {
  return min + rand() * (max - min);
}

function pick<T>(arr: T[]): T {
  return arr[Math.floor(rand() * arr.length)];
}

interface Region {
  name: string;
  latMin: number; latMax: number;
  lonMin: number; lonMax: number;
  weight: number;
}

const REGIONS: Region[] = [
  { name: 'English Channel', latMin: 49.5, latMax: 51.0, lonMin: -2.0, lonMax: 2.0, weight: 0.25 },
  { name: 'North Sea', latMin: 51.0, latMax: 56.0, lonMin: 1.0, lonMax: 7.0, weight: 0.20 },
  { name: 'Baltic', latMin: 54.0, latMax: 59.0, lonMin: 10.0, lonMax: 20.0, weight: 0.15 },
  { name: 'Mediterranean West', latMin: 36.0, latMax: 43.0, lonMin: -1.0, lonMax: 10.0, weight: 0.15 },
  { name: 'US East Coast', latMin: 36.0, latMax: 42.0, lonMin: -76.0, lonMax: -70.0, weight: 0.15 },
  { name: 'Singapore Strait', latMin: 0.5, latMax: 2.0, lonMin: 103.0, lonMax: 105.0, weight: 0.10 },
];

function pickRegion(): Region {
  const r = rand();
  let cumulative = 0;
  for (const region of REGIONS) {
    cumulative += region.weight;
    if (r <= cumulative) return region;
  }
  return REGIONS[0];
}

let vesselCache: Vessel[] | null = null;

export function generateVessels(count: number = 200): Vessel[] {
  if (vesselCache && vesselCache.length === count) return vesselCache;

  const vessels: Vessel[] = [];
  const usedMMSI = new Set<number>();
  const localRand = seededRandom(42);
  const lr = () => {
    const s = localRand();
    return s;
  };
  // Use a fresh seeded generator for consistent output
  const rng = seededRandom(42);
  const rr = (min: number, max: number) => min + rng() * (max - min);
  const pk = <T>(arr: T[]): T => arr[Math.floor(rng() * arr.length)];

  const pickReg = (): Region => {
    const r = rng();
    let c = 0;
    for (const region of REGIONS) {
      c += region.weight;
      if (r <= c) return region;
    }
    return REGIONS[0];
  };

  for (let i = 0; i < count; i++) {
    let mmsi: number;
    do {
      mmsi = 200000000 + Math.floor(rng() * 600000000);
    } while (usedMMSI.has(mmsi));
    usedMMSI.add(mmsi);

    const region = pickReg();
    const lat = rr(region.latMin, region.latMax);
    const lon = rr(region.lonMin, region.lonMax);

    vessels.push({
      mmsi,
      name: i < VESSEL_NAMES.length ? VESSEL_NAMES[i] : `Vessel-${mmsi}`,
      lat,
      lon,
      speed: Math.round(rr(0, 22) * 10) / 10,
      heading: Math.round(rr(0, 360)),
      vessel_type: pk(VESSEL_TYPES),
      timestamp: new Date(Date.now() - Math.floor(rng() * 3600000)).toISOString(),
    });
  }

  vesselCache = vessels;
  return vessels;
}

export function generateTrack(vessel: Vessel, points: number = 50): TrackPoint[] {
  const track: TrackPoint[] = [];
  const trackRng = seededRandom(vessel.mmsi);
  const rr = (min: number, max: number) => min + trackRng() * (max - min);

  let lat = vessel.lat;
  let lon = vessel.lon;
  const baseTime = new Date(vessel.timestamp).getTime();

  for (let i = points - 1; i >= 0; i--) {
    const heading = vessel.heading + rr(-15, 15);
    const headingRad = (heading * Math.PI) / 180;
    const speedKnots = vessel.speed + rr(-2, 2);
    const step = 0.005 * Math.max(0, speedKnots);

    track.push({
      mmsi: vessel.mmsi,
      lat,
      lon,
      speed: Math.round(Math.max(0, speedKnots) * 10) / 10,
      heading: Math.round(((heading % 360) + 360) % 360),
      timestamp: new Date(baseTime - i * 600000).toISOString(),
    });

    lat -= Math.cos(headingRad) * step;
    lon -= Math.sin(headingRad) * step;
  }

  return track;
}

export function vesselsToGeoJSON(vessels: Vessel[]): GeoJSON.FeatureCollection {
  return {
    type: 'FeatureCollection',
    features: vessels.map((v) => ({
      type: 'Feature' as const,
      geometry: {
        type: 'Point' as const,
        coordinates: [v.lon, v.lat],
      },
      properties: {
        mmsi: v.mmsi,
        name: v.name,
        speed: v.speed,
        heading: v.heading,
        vessel_type: v.vessel_type,
        timestamp: v.timestamp,
      },
    })),
  };
}

export function trackToGeoJSON(track: TrackPoint[]): GeoJSON.Feature {
  return {
    type: 'Feature',
    geometry: {
      type: 'LineString',
      coordinates: track.map((p) => [p.lon, p.lat]),
    },
    properties: {
      mmsi: track[0]?.mmsi,
      point_count: track.length,
    },
  };
}

export function trackPointsToGeoJSON(track: TrackPoint[]): GeoJSON.FeatureCollection {
  return {
    type: 'FeatureCollection',
    features: track.map((p) => ({
      type: 'Feature' as const,
      geometry: {
        type: 'Point' as const,
        coordinates: [p.lon, p.lat],
      },
      properties: {
        mmsi: p.mmsi,
        speed: p.speed,
        heading: p.heading,
        timestamp: p.timestamp,
      },
    })),
  };
}
