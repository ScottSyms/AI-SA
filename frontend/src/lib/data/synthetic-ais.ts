/**
 * Synthetic AIS data generator
 * Produces ~1000 ship positions and track histories spanning 90 days
 */

import type { Vessel, TrackPoint } from '$lib/platform/types';

const CARGO_NAMES = [
  'Emma Maersk', 'Ever Given', 'MSC Gulsun', 'Madrid Maersk',
  'CMA CGM Benjamin Franklin', 'OOCL Hong Kong', 'HMM Algeciras', 'MOL Triumph',
  'CMA CGM Jacques Saade', 'COSCO Shipping Nebula', 'Ever Ace', 'Ever Alot',
  'Ever Act', 'Ever Forward', 'MSC Oscar', 'Maersk Eindhoven',
  'Maersk Essen', 'Hapag-Lloyd Berlin', 'ONE Integrity', 'Hyundai Neptune',
  'Seaspan Bravo', 'APL Atlanta', 'CMA CGM Marco Polo', 'OOCL Europe',
  'Ever Gifted'
];

const TANKER_NAMES = [
  'TI Europe', 'TI Asia', 'Seawise Giant', 'Front Altair', 'Front Eagle',
  'Nave Andromeda', 'Nave Ariadne', 'DHT Tiger', 'DHT Lion', 'Almi Tankers',
  'Berge Everest', 'Suezmax Trader', 'Nordic Mistral', 'Baltic Horizon',
  'Oceanic Pride', 'Polar Endeavour', 'Aframax Star', 'Crested Falcon',
  'Ridgeway Spirit', 'Meridian Voyager'
];

const FISHING_NAMES = [
  'FV Cornelis Vrolijk', 'FV Annelies Ilena', 'FV Margiris', 'FV Saga',
  'FV Atlantic Dawn', 'FV Peterhead', 'FV Westbank', 'FV Ocean Harvest',
  'FV North Star', 'FV Silver Dawn', 'FV Sea Breeze', 'FV Northern Quest',
  'FV Arctic Hunter', 'FV Sapphire Tide', 'FV Pacific Pride', 'FV Blue Marlin',
  'FV Golden Sheaf', 'FV Sea Hunter', 'FV Harbour Light', 'FV Ocean Venture'
];

const PASSENGER_NAMES = [
  'Queen Mary 2', 'Britannia', 'Rotterdam', 'AIDAcosma', 'MSC World Europa',
  'Wonder of the Seas', 'Icon of the Seas', 'MS Europa', 'MS Eurodam',
  'Costa Smeralda', 'Carnival Vista', 'Norwegian Encore', 'Disney Dream',
  'Celebrity Edge', 'Viking Venus', 'Oasis of the Seas', 'Quantum of the Seas',
  'Iona', 'P&O Arvia', 'Aurora'
];

const TUG_NAMES = [
  'Svitzer Muir', 'Svitzer Meridian', 'Svitzer Ingrid', 'Fairplay XI',
  'Fairplay 35', 'Multratug 18', 'Multratug 19', 'Bourbon Orca',
  'Moran Explorer', 'Viking Neptune', 'Harbor Master', 'Port Assist',
  'Dock Pilot', 'Bay Tug', 'Harbor Spirit'
];

const SAILING_NAMES = [
  'Amerigo Vespucci', 'Kruzenshtern', 'Sedov', 'Statsraad Lehmkuhl',
  'Sea Cloud', 'Sea Cloud II', 'Maltese Falcon', 'Jadran', 'Etoile du Roy',
  'Christian Radich', 'Europa', 'Royal Albatross', 'Spirit of Bermuda', 'Atyla',
  'Bluenose II'
];

const PLEASURE_NAMES = [
  'A', 'Eclipse', 'Dilbar', 'Azzam', 'Rising Sun', 'Koru', 'Serene', 'Octopus',
  'Lady Moura', 'Flying Fox', 'Nero', 'Vava II', 'Al Said', 'Black Pearl',
  'Sailing Yacht A'
];

const MILITARY_NAMES = [
  'USS Gerald R. Ford', 'USS Zumwalt', 'USS Arleigh Burke', 'HMS Queen Elizabeth',
  'HMS Daring', 'INS Vikrant', 'JS Izumo', 'FS Charles de Gaulle', 'USS Nimitz',
  'HMS Prince of Wales', 'INS Kolkata', 'JS Kaga', 'USS Monterey', 'HMS Dragon',
  'FS Forbin'
];

const RESEARCH_NAMES = [
  'RV Falkor', 'RV Atlantis', 'RV Polarstern', 'RRS Sir David Attenborough',
  'NOAA Ship Okeanos Explorer', 'RV Tara', 'RV Neil Armstrong', 'RV Investigator',
  'RV Metops', 'RV Sonne', 'RV Sikuliaq', 'RV Maria S. Merian', 'RV Revelle',
  'RV Pelagia', 'RV Calypso'
];

const PILOT_NAMES = [
  'Pilot 1', 'Pilot 2', 'Pilot 3', 'Pilot 4', 'Pilot 5', 'Pilot 6', 'Pilot 7',
  'Pilot 8', 'Pilot 9', 'Pilot 10', 'Port Pilot', 'Harbor Pilot', 'Sea Pilot',
  'Dock Pilot', 'Channel Pilot'
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

function namesForType(vesselType: string): string[] {
  switch (vesselType) {
    case 'Cargo': return CARGO_NAMES;
    case 'Tanker': return TANKER_NAMES;
    case 'Fishing': return FISHING_NAMES;
    case 'Passenger': return PASSENGER_NAMES;
    case 'Tug': return TUG_NAMES;
    case 'Sailing': return SAILING_NAMES;
    case 'Pleasure': return PLEASURE_NAMES;
    case 'Military': return MILITARY_NAMES;
    case 'Research': return RESEARCH_NAMES;
    case 'Pilot': return PILOT_NAMES;
    default: return CARGO_NAMES;
  }
}

function pickVesselName(vesselType: string, index: number): string {
  const names = namesForType(vesselType);
  return index < names.length ? names[index] : `${vesselType} ${index + 1}`;
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

export function generateVessels(count: number = 1000): Vessel[] {
  if (vesselCache && vesselCache.length === count) return vesselCache;

  const vessels: Vessel[] = [];
  const usedMMSI = new Set<number>();
  const vesselNameCounts = new Map<string, number>();
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

  for (let _i = 0; _i < count; _i++) {
    let mmsi: number;
    do {
      mmsi = 200000000 + Math.floor(rng() * 600000000);
    } while (usedMMSI.has(mmsi));
    usedMMSI.add(mmsi);

    const region = pickReg();
    const lat = rr(region.latMin, region.latMax);
    const lon = rr(region.lonMin, region.lonMax);
    const vesselType = pk(VESSEL_TYPES);
    const nameIndex = vesselNameCounts.get(vesselType) ?? 0;
    vesselNameCounts.set(vesselType, nameIndex + 1);

    vessels.push({
      mmsi,
      name: pickVesselName(vesselType, nameIndex),
      lat,
      lon,
      speed: Math.round(rr(0, 22) * 10) / 10,
      heading: Math.round(rr(0, 360)),
      vessel_type: vesselType,
      timestamp: new Date().toISOString(),
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
  const endTime = new Date(vessel.timestamp).getTime();
  const startTime = endTime - TRACK_SPAN_DAYS * 24 * 60 * 60 * 1000;
  const denominator = Math.max(points - 1, 1);

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
      timestamp: new Date(startTime + ((TRACK_SPAN_DAYS * 24 * 60 * 60 * 1000) * i) / denominator).toISOString(),
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
        lat: v.lat,
        lon: v.lon,
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
const TRACK_SPAN_DAYS = 90;
