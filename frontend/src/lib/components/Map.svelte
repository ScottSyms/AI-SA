<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import maplibregl from 'maplibre-gl';
  import { mapStore } from '$lib/platform/map/store';
  import { drawModeStore, setDrawMode } from '$lib/platform/map/draw-mode';
  import {
    selectionStore,
    selectVessel,
    clearSelection,
    selectByGeometry,
  } from '$lib/platform/selection/store';
  import {
    generateVessels,
    vesselsToGeoJSON,
  } from '$lib/data/synthetic-ais';
  import {
    initDuckDB,
    queryVesselsInPolygon,
    queryVesselsInRadius,
  } from '$lib/platform/duckdb/client';
  import type { Vessel, SelectionItem } from '$lib/platform/types';
  import { agentResultStore, agentShouldShowMap } from '$lib/platform/render/store';
  import { fetchSkills, fetchSkillLayerData, type BackendSkill } from '$lib/platform/api/client';

  let mapContainer: HTMLDivElement;
  let map: maplibregl.Map;
  const vessels = generateVessels(200);
  const vesselsByMMSI = new Map<number, Vessel>();
  vessels.forEach((v) => vesselsByMMSI.set(v.mmsi, v));

  // Drawing state
  let drawingPoints: [number, number][] = [];
  let isDrawing = false;
  let radiusCenter: [number, number] | null = null;
  let currentDrawMode: string = 'none';

  const VESSEL_TYPE_COLORS: Record<string, string> = {
    Cargo: '#4A90D9',
    Tanker: '#E74C3C',
    Fishing: '#27AE60',
    Passenger: '#F39C12',
    Tug: '#8E44AD',
    Sailing: '#1ABC9C',
    Pleasure: '#F1C40F',
    Military: '#2C3E50',
    Research: '#3498DB',
    Pilot: '#E67E22',
  };

  function buildColorExpression(): maplibregl.ExpressionSpecification {
    const expr: unknown[] = ['match', ['get', 'vessel_type']];
    for (const [type, color] of Object.entries(VESSEL_TYPE_COLORS)) {
      expr.push(type, color);
    }
    expr.push('#888888');
    return expr as maplibregl.ExpressionSpecification;
  }

  function clearTrack() {
    if (!map || !map.getSource('vessel-track')) return;
    const trackSource = map.getSource('vessel-track') as maplibregl.GeoJSONSource;
    const trackPointsSource = map.getSource('vessel-track-points') as maplibregl.GeoJSONSource;

    trackSource.setData({ type: 'FeatureCollection', features: [] });
    trackPointsSource.setData({ type: 'FeatureCollection', features: [] });
  }

  function updateHighlight(mmsis: number[]) {
    if (!map || !map.getLayer('ship-positions-highlight')) return;
    map.setFilter('ship-positions-highlight', [
      'in',
      ['get', 'mmsi'],
      ['literal', mmsis],
    ]);
  }

  // --- Drawing helpers ---

  function updateDrawingLayer() {
    if (!map || !map.getSource('drawing')) return;
    const src = map.getSource('drawing') as maplibregl.GeoJSONSource;

    if (drawingPoints.length < 2) {
      src.setData({ type: 'FeatureCollection', features: [] });
      return;
    }

    const features: GeoJSON.Feature[] = [];

    if (currentDrawMode === 'polygon') {
      const coords = [...drawingPoints];
      if (coords.length >= 3) {
        coords.push(coords[0]); // close ring
      }
      features.push({
        type: 'Feature',
        geometry: { type: 'Polygon', coordinates: [coords] },
        properties: {},
      });
      // Also draw the line for visual feedback
      features.push({
        type: 'Feature',
        geometry: { type: 'LineString', coordinates: coords },
        properties: {},
      });
    }

    src.setData({ type: 'FeatureCollection', features });
  }

  function updateRadiusLayer(center: [number, number], radiusKm: number) {
    if (!map || !map.getSource('drawing')) return;
    const src = map.getSource('drawing') as maplibregl.GeoJSONSource;

    // Generate circle polygon (64 segments)
    const points: [number, number][] = [];
    const segments = 64;
    for (let i = 0; i <= segments; i++) {
      const angle = (i / segments) * 2 * Math.PI;
      const latDelta = (radiusKm / 111.32) * Math.cos(angle);
      const lonDelta = (radiusKm / (111.32 * Math.cos((center[1] * Math.PI) / 180))) * Math.sin(angle);
      points.push([center[0] + lonDelta, center[1] + latDelta]);
    }

    src.setData({
      type: 'FeatureCollection',
      features: [
        {
          type: 'Feature',
          geometry: { type: 'Polygon', coordinates: [points] },
          properties: {},
        },
        {
          type: 'Feature',
          geometry: { type: 'Point', coordinates: center },
          properties: {},
        },
      ],
    });
  }

  function clearDrawing() {
    drawingPoints = [];
    isDrawing = false;
    radiusCenter = null;
    if (map && map.getSource('drawing')) {
      (map.getSource('drawing') as maplibregl.GeoJSONSource).setData({
        type: 'FeatureCollection',
        features: [],
      });
    }
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

  async function finishPolygonDraw() {
    if (drawingPoints.length < 3) {
      clearDrawing();
      return;
    }

    const polygon = [...drawingPoints, drawingPoints[0]]; // close ring
    try {
      const results = await queryVesselsInPolygon(polygon);
      const items: SelectionItem[] = results.map((r) => ({
        mmsi: Number(r.mmsi),
        name: String(r.name),
        speed: r.speed,
        heading: r.heading,
        vessel_type: r.vessel_type,
        timestamp: r.timestamp,
        lat: r.lat,
        lon: r.lon,
      }));

      if (items.length > 0) {
        selectByGeometry(items, {
          type: 'Polygon',
          coordinates: [polygon],
        });
      }
    } catch (err) {
      console.error('[Draw] Polygon query failed:', err);
    }

    clearDrawing();
    setDrawMode('none');
  }

  async function finishRadiusDraw(center: [number, number], radiusKm: number) {
    try {
      const results = await queryVesselsInRadius(center[0], center[1], radiusKm);
      const items: SelectionItem[] = results.map((r) => ({
        mmsi: Number(r.mmsi),
        name: String(r.name),
        speed: r.speed,
        heading: r.heading,
        vessel_type: r.vessel_type,
        timestamp: r.timestamp,
        lat: r.lat,
        lon: r.lon,
      }));

      if (items.length > 0) {
        selectByGeometry(items, {
          type: 'Point',
          coordinates: center,
        });
      }
    } catch (err) {
      console.error('[Draw] Radius query failed:', err);
    }

    clearDrawing();
    setDrawMode('none');
  }

  onMount(() => {
    map = new maplibregl.Map({
      container: mapContainer,
      style: {
        version: 8,
        name: 'Geospatial Platform',
        sources: {
          'osm-tiles': {
            type: 'raster',
            tiles: ['https://tile.openstreetmap.org/{z}/{x}/{y}.png'],
            tileSize: 256,
            attribution:
              '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors',
          },
        },
        layers: [
          {
            id: 'osm-base',
            type: 'raster',
            source: 'osm-tiles',
            minzoom: 0,
            maxzoom: 19,
          },
        ],
      },
      center: [2.0, 50.0],
      zoom: 5,
    });

    map.addControl(new maplibregl.NavigationControl(), 'top-right');

    map.on('load', () => {
      mapStore.set(map);

      // --- Vessel positions ---
      map.addSource('vessels', {
        type: 'geojson',
        data: vesselsToGeoJSON(vessels),
      });

      map.addLayer({
        id: 'ship-positions',
        type: 'circle',
        source: 'vessels',
        paint: {
          'circle-radius': ['interpolate', ['linear'], ['zoom'], 3, 3, 8, 6, 12, 10],
          'circle-color': buildColorExpression(),
          'circle-stroke-color': '#ffffff',
          'circle-stroke-width': 1,
          'circle-opacity': 0.85,
        },
      });

      map.addLayer({
        id: 'ship-positions-highlight',
        type: 'circle',
        source: 'vessels',
        filter: ['in', ['get', 'mmsi'], ['literal', []]],
        paint: {
          'circle-radius': ['interpolate', ['linear'], ['zoom'], 3, 6, 8, 10, 12, 14],
          'circle-color': buildColorExpression(),
          'circle-stroke-color': '#FFD700',
          'circle-stroke-width': 3,
          'circle-opacity': 1,
        },
      });

      // --- Track ---
      map.addSource('vessel-track', {
        type: 'geojson',
        data: { type: 'FeatureCollection', features: [] },
      });

      map.addSource('vessel-track-points', {
        type: 'geojson',
        data: { type: 'FeatureCollection', features: [] },
      });

      map.addLayer({
        id: 'vessel-track-line',
        type: 'line',
        source: 'vessel-track',
        paint: {
          'line-color': '#FFD700',
          'line-width': 2,
          'line-opacity': 0.8,
          'line-dasharray': [2, 2],
        },
      });

      map.addLayer({
        id: 'vessel-track-dots',
        type: 'circle',
        source: 'vessel-track-points',
        paint: {
          'circle-radius': 3,
          'circle-color': '#FFD700',
          'circle-opacity': 0.6,
        },
      });

      // --- Drawing layer ---
      map.addSource('drawing', {
        type: 'geojson',
        data: { type: 'FeatureCollection', features: [] },
      });

      map.addLayer({
        id: 'drawing-fill',
        type: 'fill',
        source: 'drawing',
        filter: ['==', '$type', 'Polygon'],
        paint: {
          'fill-color': '#FFD700',
          'fill-opacity': 0.1,
        },
      });

      map.addLayer({
        id: 'drawing-line',
        type: 'line',
        source: 'drawing',
        filter: ['in', '$type', 'Polygon', 'LineString'],
        paint: {
          'line-color': '#FFD700',
          'line-width': 2,
          'line-dasharray': [3, 2],
        },
      });

      map.addLayer({
        id: 'drawing-point',
        type: 'circle',
        source: 'drawing',
        filter: ['==', '$type', 'Point'],
        paint: {
          'circle-radius': 5,
          'circle-color': '#FFD700',
          'circle-stroke-color': '#fff',
          'circle-stroke-width': 2,
        },
      });

      // --- Agent result overlay ---
      map.addSource('agent-overlay', {
        type: 'geojson',
        data: { type: 'FeatureCollection', features: [] },
      });

      map.addLayer({
        id: 'agent-overlay-circle',
        type: 'circle',
        source: 'agent-overlay',
        filter: ['==', '$type', 'Point'],
        paint: {
          'circle-radius': ['interpolate', ['linear'], ['zoom'], 3, 5, 8, 9, 12, 13],
          'circle-color': '#00FFAA',
          'circle-stroke-color': '#ffffff',
          'circle-stroke-width': 2,
          'circle-opacity': 0.9,
        },
      });

      map.addLayer({
        id: 'agent-overlay-line',
        type: 'line',
        source: 'agent-overlay',
        filter: ['==', '$type', 'LineString'],
        paint: {
          'line-color': '#00FFAA',
          'line-width': 3,
          'line-opacity': 0.8,
        },
      });

      map.addLayer({
        id: 'agent-overlay-fill',
        type: 'fill',
        source: 'agent-overlay',
        filter: ['==', '$type', 'Polygon'],
        paint: {
          'fill-color': '#00FFAA',
          'fill-opacity': 0.15,
        },
      });

      // --- Click handler ---
      map.on('click', (e) => {
        if (currentDrawMode === 'polygon') {
          drawingPoints.push([e.lngLat.lng, e.lngLat.lat]);
          isDrawing = true;
          updateDrawingLayer();
          return;
        }

        if (currentDrawMode === 'radius') {
          if (!radiusCenter) {
            radiusCenter = [e.lngLat.lng, e.lngLat.lat];
            return;
          }
          // Second click completes radius
          const radiusKm = haversineKm(
            radiusCenter[1], radiusCenter[0],
            e.lngLat.lat, e.lngLat.lng
          );
          finishRadiusDraw(radiusCenter, radiusKm);
          return;
        }

        // Normal select mode
        const features = map.queryRenderedFeatures(e.point, {
          layers: ['ship-positions'],
        });

        if (features && features.length > 0) {
          const props = features[0].properties;
          if (props) {
            const mmsi = Number(props.mmsi);
            const vessel = vesselsByMMSI.get(mmsi);
            const selectionItem: SelectionItem = vessel
              ? { ...vessel }
              : {
                  mmsi,
                  name: String(props.name),
                  lat: Number(props.lat),
                  lon: Number(props.lon),
                  speed: Number(props.speed),
                  heading: Number(props.heading),
                  vessel_type: String(props.vessel_type),
                  timestamp: String(props.timestamp),
                };
            selectVessel(
              selectionItem,
              e.originalEvent.shiftKey
            );
          }
        } else {
          clearSelection();
        }
      });

      // Double-click to finish polygon
      map.on('dblclick', (e) => {
        if (currentDrawMode === 'polygon' && drawingPoints.length >= 3) {
          e.preventDefault();
          finishPolygonDraw();
        }
      });

      // Mouse move for radius preview
      map.on('mousemove', (e) => {
        if (currentDrawMode === 'radius' && radiusCenter) {
          const radiusKm = haversineKm(
            radiusCenter[1], radiusCenter[0],
            e.lngLat.lat, e.lngLat.lng
          );
          updateRadiusLayer(radiusCenter, radiusKm);
        }
      });

      // Cursor styles
      map.on('mouseenter', 'ship-positions', () => {
        if (currentDrawMode === 'none') {
          map.getCanvas().style.cursor = 'pointer';
        }
      });
      map.on('mouseleave', 'ship-positions', () => {
        if (currentDrawMode === 'none') {
          map.getCanvas().style.cursor = '';
        }
      });

      // Initialize DuckDB after map loads
      initDuckDB(vessels);

      // Dynamically load skill layers from backend
      loadSkillLayers(map);
    });
  });

  /** Fetch skills from backend and register their data as map layers. */
  async function loadSkillLayers(mapInstance: maplibregl.Map) {
    try {
      const skills = await fetchSkills();
      for (const skill of skills) {
        if (!skill.map_layers || skill.map_layers.length === 0) continue;

        // Skip ais_positions — already rendered client-side from synthetic data
        if (skill.name === 'ais_positions') continue;

        const layerData = await fetchSkillLayerData(skill.name);
        if (!layerData || layerData.feature_count === 0) continue;

        const sourceId = `skill-${skill.name}`;
        mapInstance.addSource(sourceId, {
          type: 'geojson',
          data: layerData.geojson,
        });

        // Register each declared map layer
        for (const layerDef of skill.map_layers) {
          const layerId = `skill-${skill.name}-${layerDef.id}`;

          if (layerDef.type === 'circle') {
            mapInstance.addLayer({
              id: layerId,
              type: 'circle',
              source: sourceId,
              paint: {
                'circle-radius': ['interpolate', ['linear'], ['zoom'], 3, 4, 8, 7, 12, 11],
                'circle-color': '#FF6B35',
                'circle-stroke-color': '#ffffff',
                'circle-stroke-width': 1.5,
                'circle-opacity': 0.9,
              },
            });
          } else if (layerDef.type === 'symbol') {
            mapInstance.addLayer({
              id: layerId,
              type: 'symbol',
              source: sourceId,
              layout: {
                'text-field': ['get', 'port_name'],
                'text-size': 11,
                'text-offset': [0, 1.5],
                'text-anchor': 'top',
                'text-optional': true,
              },
              paint: {
                'text-color': '#333333',
                'text-halo-color': '#ffffff',
                'text-halo-width': 1.5,
              },
              minzoom: 6,
            });
          } else if (layerDef.type === 'line') {
            mapInstance.addLayer({
              id: layerId,
              type: 'line',
              source: sourceId,
              paint: {
                'line-color': '#FF6B35',
                'line-width': 2,
                'line-opacity': 0.8,
              },
            });
          } else if (layerDef.type === 'fill') {
            mapInstance.addLayer({
              id: layerId,
              type: 'fill',
              source: sourceId,
              paint: {
                'fill-color': '#FF6B35',
                'fill-opacity': 0.15,
              },
            });
          }
        }

        console.log(`[Map] Loaded skill layer: ${skill.name} (${layerData.feature_count} features)`);
      }
    } catch (err) {
      console.warn('[Map] Failed to load skill layers (backend may be offline):', err);
    }
  }

  // Subscribe to draw mode changes
  const unsubDrawMode = drawModeStore.subscribe((mode) => {
    currentDrawMode = mode;
    clearDrawing();
    if (map) {
      if (mode === 'polygon') {
        map.getCanvas().style.cursor = 'crosshair';
        map.doubleClickZoom.disable();
      } else if (mode === 'radius') {
        map.getCanvas().style.cursor = 'crosshair';
      } else {
        map.getCanvas().style.cursor = '';
        map.doubleClickZoom.enable();
      }
    }
  });

  // React to selection changes
  const unsubSelection = selectionStore.subscribe((state) => {
    const mmsis = state.items.map((i) => i.mmsi);
    updateHighlight(mmsis);
    clearTrack();
  });

  // React to agent result geometry
  const unsubAgentResult = agentResultStore.subscribe((result) => {
    if (!map || !map.getSource('agent-overlay')) return;
    const src = map.getSource('agent-overlay') as maplibregl.GeoJSONSource;

    if (result.status === 'ok' && result.render_as.includes('map') && result.geometry) {
      src.setData(result.geometry as GeoJSON.GeoJSON);

      // Fit map to show all agent result features
      if (result.geometry.features.length > 0) {
        const bounds = new maplibregl.LngLatBounds();
        for (const feature of result.geometry.features) {
          const geom = feature.geometry;
          if (geom.type === 'Point') {
            bounds.extend(geom.coordinates as [number, number]);
          }
        }
        if (!bounds.isEmpty()) {
          map.fitBounds(bounds, { padding: 80, maxZoom: 12, duration: 1000 });
        }
      }
    } else {
      src.setData({ type: 'FeatureCollection', features: [] });
    }
  });

  // Escape key cancels drawing
  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      if (currentDrawMode !== 'none') {
        clearDrawing();
        setDrawMode('none');
      } else {
        clearSelection();
      }
    }
  }

  onDestroy(() => {
    unsubDrawMode();
    unsubSelection();
    unsubAgentResult();
    if (map) {
      mapStore.set(null);
      map.remove();
    }
  });
</script>

<svelte:window onkeydown={handleKeydown} />

<div bind:this={mapContainer} class="map-container"></div>

<style>
  .map-container {
    position: relative;
    width: 100%;
    height: 100%;
    min-height: 0;
  }
</style>
