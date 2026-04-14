---
skill: ais_positions
source_type: parquet
source_path: data/seed/**/*.parquet

interactions:
  - multi_select
  - lasso_select
  - radius_query
  - attribute_filter
  - viewport_filter

map_layers:
  - id: ship_positions
    type: circle
  - id: ship_track
    type: line
---

## Source

AIS (Automatic Identification System) vessel position data.
Loaded from parquet files in `data/seed/`. Each record represents
a vessel position report with MMSI, coordinates, speed, heading,
vessel type, and timestamp.

## Tools

### get_vessel_info
Returns details about a specific vessel by MMSI.
- Parameters: `mmsi` (integer)
- SQL: `vessel_info.sql`

### get_vessel_track
Returns the track history for a vessel.
- Parameters: `mmsi` (integer), `limit` (integer, default 100)
- SQL: `vessel_track.sql`

### get_latest_positions
Returns the latest position for all vessels, optionally filtered.
- Parameters: `vessel_type` (string, optional), `limit` (integer, default 200)
- SQL: `latest_positions.sql`

### get_vessels_in_bbox
Returns vessels within a bounding box.
- Parameters: `min_lon`, `min_lat`, `max_lon`, `max_lat` (floats)
- SQL: `vessels_in_bbox.sql`

## Domain prompt

You are an AIS maritime intelligence analyst. You help users understand
vessel positions, movements, and patterns.

When a user selects a vessel, provide its identity, type, speed, and heading.
When asked about tracks, show the vessel's movement history.
When a spatial selection is made, summarize the vessels within the area.

Always reference vessels by name and MMSI.
Use nautical terminology where appropriate (knots, heading, bearing).
Flag unusual behavior: stationary cargo vessels, high-speed fishing vessels,
vessels in unexpected areas.

## SQL views

- `v_latest_positions` — most recent position per vessel
- `v_vessel_tracks` — all position reports ordered by time
- `v_vessel_summary` — aggregated stats per vessel
