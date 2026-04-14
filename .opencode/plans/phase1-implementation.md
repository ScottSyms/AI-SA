# Phase 1 Implementation Plan — Geospatial Intelligence Platform

## Tech Stack
- **Frontend**: SvelteKit + TypeScript (already scaffolded)
- **Map**: MapLibre GL JS (installed)
- **Drawing**: @mapbox/mapbox-gl-draw (installed)
- **Backend**: Rust (Axum) — skeleton only in Phase 1
- **DuckDB**: WASM (browser) + server-side (Rust) — Phase 2+
- **LLM**: OpenAI (abstracted) — Phase 4
- **Tiles**: Protomaps
- **Package mgr**: pnpm

## Status
- SvelteKit project scaffolded at `frontend/`
- `maplibre-gl` and `@mapbox/mapbox-gl-draw` installed
- Directory structure created: `skills/ais_positions/sql/`, `data/seed/`, `backend/src/`

## Phase 1 Files to Create

### 1. Core Types (`frontend/src/lib/platform/types.ts`)
- `RenderEnvelope` — normalized output contract (§4)
- `SelectionState`, `SelectionItem` — selection model (§10)
- `SkillManifest`, `ToolDef` — skill system types (§5)
- `Vessel`, `TrackPoint` — AIS domain types

### 2. Synthetic Data (`frontend/src/lib/data/synthetic-ais.ts`)
- `generateVessels(count=200)` — deterministic PRNG, ~200 ships across 6 maritime regions
  - English Channel, North Sea, Baltic, Mediterranean, US East Coast, Singapore Strait
- `generateTrack(vessel, points=50)` — walk-back track history
- `vesselsToGeoJSON()`, `trackToGeoJSON()`, `trackPointsToGeoJSON()` — converters
- 50 named vessels, 10 vessel types, seeded random for reproducibility

### 3. Selection Store (`frontend/src/lib/platform/selection/store.ts`)
- Svelte writable store: `selectionStore`
- Derived: `selectedItems`, `primarySelection`, `hasSelection`, `selectionCount`
- Functions: `selectVessel(item, multi)`, `clearSelection()`, `selectByGeometry(items, geom)`
- Implements §10 context model and §11 platform interaction tools

### 4. Map Store (`frontend/src/lib/platform/map/store.ts`)
- Writable store holding MapLibre map instance for cross-component access

### 5. Map Component (`frontend/src/lib/components/Map.svelte`)
- Full-viewport MapLibre GL JS map
- Protomaps basemap via PMTiles protocol
- On load:
  - Add `vessels` GeoJSON source from synthetic data
  - Add `ship-positions` circle layer (color by vessel_type, size responsive)
  - Add `ship-positions-highlight` layer for selected features
  - Add `vessel-track` line source/layer (initially empty)
- Click handler:
  - Query rendered features under click
  - Call `selectVessel()` (shift = multi)
  - On select: generate track, update track source
  - On deselect: clear track
- Highlight: filter expression on selected MMSIs
- Export map instance to `mapStore`

### 6. Detail Panel (`frontend/src/lib/components/DetailPanel.svelte`)
- Slide-out panel (right side) reactive to `primarySelection`
- Shows: name, MMSI, vessel type, speed, heading, coordinates, timestamp
- Close button calls `clearSelection()`
- Styled with CSS transitions

### 7. Command Bar (`frontend/src/lib/components/CommandBar.svelte`)
- Fixed bottom text input
- Stub: captures text, logs to console
- Visual presence for future agent integration

### 8. Table View (`frontend/src/lib/components/TableView.svelte`)
- Stub component — collapsible panel showing selected vessels in tabular form
- Reactive to `selectedItems`

### 9. Layout (`frontend/src/routes/+layout.svelte`)
- Global CSS reset, full-viewport layout
- Import MapLibre CSS

### 10. Main Page (`frontend/src/routes/+page.svelte`)
- Compose: Map + DetailPanel + CommandBar + TableView
- Full-viewport map as base layer, panels overlay

### 11. App HTML (`frontend/src/app.html`)
- Add title: "Geospatial Intelligence Platform"

### 12. Skill Manifest (`skills/ais_positions/skill.md`)
- YAML front matter per §5
- Sections: Source, Tools, Domain prompt, SQL views
- Map layers: `ship_positions` (circle), `ship_track` (line)

### 13. SQL Files
- `skills/ais_positions/sql/views.sql` — CREATE VIEW for latest positions
- `skills/ais_positions/sql/latest_positions.sql` — query latest position per vessel
- `skills/ais_positions/sql/vessel_track.sql` — query track for a given MMSI
- `skills/ais_positions/sql/vessels_in_bbox.sql` — spatial bounding box filter

### 14. Backend Skeleton (`backend/`)
- `Cargo.toml` — axum, tokio, serde, duckdb deps
- `src/main.rs` — Hello world Axum server with `/api/health`

### 15. Root Files
- `.gitignore` — node_modules, target, .env, .svelte-kit, etc.
- `.env.example` — OPENAI_API_KEY placeholder

## Vessel Type Color Scheme
| Type | Color |
|---|---|
| Cargo | #4A90D9 |
| Tanker | #E74C3C |
| Fishing | #27AE60 |
| Passenger | #F39C12 |
| Tug | #8E44AD |
| Sailing | #1ABC9C |
| Pleasure | #F1C40F |
| Military | #2C3E50 |
| Research | #3498DB |
| Pilot | #E67E22 |

## Key Architecture Patterns
1. **No inline SQL** — all SQL in `/skills/{skill}/sql/*.sql`
2. **No domain logic in platform** — vessel rendering driven by skill manifest
3. **Selection store is platform-owned** — components react to it
4. **Render envelope** — all outputs flow through `RenderEnvelope` type
5. **MapLibre layers driven by data** — no hardcoded feature rendering

## Phase 1 Definition of Done
- Map displays with Protomaps basemap
- ~200 ship markers visible across maritime regions
- Click ship → detail panel shows vessel info
- Shift+click → multi-select
- Click ship → track renders as line on map
- Click elsewhere → deselect, track clears
- Command bar visible (non-functional)
- All types and stores in place for Phase 2

## Next Phases (not in scope)
- **Phase 2**: DuckDB-WASM, drawing tools, table view, spatial queries
- **Phase 3**: Rust Axum backend, skill loader, server DuckDB
- **Phase 4**: Agent runtime, LLM integration, command bar wiring
- **Phase 5**: Voice pipeline, ad hoc SQL, second skill proof
