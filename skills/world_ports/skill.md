---
skill: world_ports
source_type: parquet
source_path: data/seed/world_ports.parquet

interactions:
  - multi_select
  - radius_query
  - attribute_filter
  - viewport_filter

map_layers:
  - id: port_markers
    type: circle
  - id: port_labels
    type: symbol
---

## Source

World ports and harbors reference data. Each record represents a port
with its name, country, coordinates, size classification, and facilities.
Loaded from parquet file in `data/seed/`.

## Tools

### get_port_info
Returns details about a specific port by port_id.
- Parameters: `port_id` (integer, required)
- SQL: `port_info.sql`

### search_ports
Search ports by name or country.
- Parameters: `search` (string, required), `limit` (integer, default 20)
- SQL: `search_ports.sql`

### get_ports_in_bbox
Returns ports within a bounding box.
- Parameters: `min_lon` (float, required), `min_lat` (float, required), `max_lon` (float, required), `max_lat` (float, required), `limit` (integer, default 100)
- SQL: `ports_in_bbox.sql`

### get_ports_by_size
Returns ports filtered by size classification.
- Parameters: `size` (string, required), `limit` (integer, default 50)
- SQL: `ports_by_size.sql`

## Domain prompt

You are a port and harbor reference analyst. You help users find and explore
world ports and their facilities.

When a user asks about a port, provide its full details: name, country,
coordinates, size, and available facilities (cargo, tanker, container, etc.).

When asked to find ports near a location, use proximity queries with
Euclidean distance approximation (no spatial extensions available).

Port sizes: Large, Medium, Small, Very Small.
Facility types: cargo, tanker, container, fishing, passenger, repair, drydock.

## SQL views

- `v_all_ports` -- all ports with full details
- `v_port_summary` -- port counts by country and size
