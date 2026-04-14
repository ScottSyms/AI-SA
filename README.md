# Tommy3

Skill-driven geospatial intelligence platform built on MapLibre GL JS, DuckDB, and an LLM-powered agent runtime.

Tommy3 is a **platform, not an application**. The platform owns rendering, interaction, agent orchestration, and tool execution. **Skills** are drop-in domain modules that provide data sources, SQL logic, tool contracts, and map layer definitions — no platform code changes required.

## Architecture

```
┌─────────────────────────────────────────────────────┐
│  Frontend (SvelteKit + Svelte 5 + MapLibre GL JS)   │
│  ┌───────────┐ ┌──────────┐ ┌───────┐ ┌─────────┐  │
│  │ Map       │ │ TableView│ │ Panel │ │ Command │  │
│  │ (MapLibre)│ │          │ │       │ │ Bar     │  │
│  └───────────┘ └──────────┘ └───────┘ └─────────┘  │
│  DuckDB-WASM (client-side spatial queries)          │
├─────────────────────────────────────────────────────┤
│  Vite proxy /api/* → :3001                          │
├─────────────────────────────────────────────────────┤
│  Backend (Rust Axum)                                │
│  ┌───────────┐ ┌──────────┐ ┌───────────────────┐  │
│  │ Agent     │ │ Skill    │ │ DuckDB (bundled)  │  │
│  │ Runtime   │ │ Loader   │ │                   │  │
│  └───────────┘ └──────────┘ └───────────────────┘  │
├─────────────────────────────────────────────────────┤
│  Skills (drop-in directories)                       │
│  ┌────────────────┐  ┌────────────────┐             │
│  │ ais_positions   │  │ world_ports    │  │ ...     │
│  │ skill.md + sql/ │  │ skill.md + sql/│  │         │
│  └────────────────┘  └────────────────┘             │
└─────────────────────────────────────────────────────┘
```

## Tech Stack

| Layer | Technology | Notes |
|-------|-----------|-------|
| Frontend | SvelteKit, Svelte 5 (runes), TypeScript | pnpm, Vite 8 |
| Map | MapLibre GL JS 5 | Custom draw controls for polygon/radius |
| Client DB | DuckDB-WASM | Synthetic AIS data for map interactions |
| Backend | Rust (edition 2024), Axum 0.8 | Requires Rust 1.85+ |
| Server DB | DuckDB 1.2 (bundled) | Parquet files, registered SQL views |
| LLM | OpenAI (gpt-4o-mini default) | Function-calling agent with tool loop |
| Map tiles | OSM raster (interim) | Protomaps/pmtiles ready for vector tiles |

## Prerequisites

- **Rust 1.85+** (for edition 2024)
- **Node.js** and **pnpm**
- **OpenAI API key**

## Quick Start

```bash
# 1. Clone and configure
cp .env.example .env   # or create .env with OPENAI_API_KEY=sk-...

# 2. Generate seed data
cd backend
cargo run --bin seed-data     # → data/seed/ais_sample.parquet (10K rows)
cargo run --bin seed-ports    # → data/seed/world_ports.parquet (81 ports)

# 3. Start the backend (port 3001)
cargo run --bin tommy3-backend

# 4. Start the frontend (port 5174) — in another terminal
cd frontend
pnpm install
pnpm dev
```

Open **http://localhost:5174** in a browser.

## Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `OPENAI_API_KEY` | Yes | — | OpenAI API key for agent queries |
| `OPENAI_MODEL` | No | `gpt-4o-mini` | Model to use for agent |
| `BIND_ADDR` | No | `0.0.0.0:3001` | Backend listen address |

## Interaction Modes

All three are first-class and feed the same agent pipeline:

- **Mouse** — Click vessels, multi-select, draw polygons, draw radius circles, viewport filtering
- **Text** — Type queries in the command bar (e.g., "show tankers near Vancouver")
- **Voice** — Click the mic button; speech is transcribed and sent through the same pipeline as text

## Skill System

Skills are drop-in domain modules in `/skills/{name}/`. Each skill has:

```
skills/ais_positions/
├── skill.md          # YAML front matter + markdown sections
└── sql/
    ├── get_vessel_info.sql
    ├── get_vessel_track.sql
    ├── get_latest_positions.sql
    ├── get_vessels_in_bbox.sql
    └── views.sql
```

### skill.md Structure

```yaml
---
skill: ais_positions
source_type: parquet
source_path: data/seed/**/*.parquet
interactions:
  - multi_select
  - lasso_select
  - radius_query
map_layers:
  - id: ship_positions
    type: circle
---

## Source
Description of data source and schema.

## Tools
Tool declarations with parameters and SQL file mappings.

## Domain prompt
Injected into the LLM system prompt for domain reasoning.

## SQL views
DuckDB views registered at startup.
```

### Boundary Rules

| Concern | Owner |
|---------|-------|
| SQL logic | Skill |
| Tool signature | Skill |
| Tool execution | Platform |
| Rendering | Platform |
| Map layer definition | Skill |
| Map layer rendering | Platform |
| Interaction mechanics | Platform |
| Domain reasoning | Skill prompt |

### Included Skills

| Skill | Data | Tools | Map Layers |
|-------|------|-------|------------|
| `ais_positions` | AIS vessel positions (200 vessels, 10K rows) | `get_vessel_info`, `get_vessel_track`, `get_latest_positions`, `get_vessels_in_bbox` | ship_positions (circle), ship_track (line) |
| `world_ports` | World ports/harbors (81 ports) | `get_port_info`, `search_ports`, `get_ports_in_bbox`, `get_ports_by_size` | port_markers (circle), port_labels (symbol) |

## DuckDB: Dual Deployment

Tommy3 runs **two independent DuckDB instances** by design:

- **Browser (WASM)**: Holds synthetic AIS data generated client-side. Powers map interactions (click, select, draw queries) with zero network latency.
- **Server (bundled)**: Loads parquet files and registers skill SQL views. Powers the agent's `run_sql` tool and skill tool execution.

Neither instance has the spatial extension. Spatial queries use:
- Bounding-box SQL (`WHERE lat BETWEEN ... AND lon BETWEEN ...`)
- Haversine formula for accurate distances
- Client-side ray-casting for point-in-polygon

## Agent Runtime

The LLM agent uses OpenAI function-calling with:
- Dynamic system prompt built from live DuckDB schema + skill domain prompts
- `run_sql` tool for ad hoc SELECT queries (validated, read-only, LIMIT enforced)
- Skill-declared tools mapped to SQL files
- Max 8 tool-calling rounds with final-round text-only fallback
- All results normalized to a **RenderEnvelope**:

```json
{
  "status": "ok",
  "title": "",
  "summary": "",
  "data": {},
  "render_as": ["map", "table"],
  "columns": [],
  "geometry": {},
  "actions": [],
  "warnings": []
}
```

## Project Structure

```
tommy3/
├── AGENTS.md              # Architecture specification
├── README.md              # This file
├── .env                   # Environment variables
├── frontend/              # SvelteKit app (port 5174)
│   └── src/lib/
│       ├── platform/      # Types, API client, stores
│       ├── components/    # Map, CommandBar, TableView, DetailPanel, Toolbar
│       └── data/          # Synthetic AIS generator
├── backend/               # Rust Axum server (port 3001)
│   └── src/
│       ├── main.rs        # Router, startup
│       ├── agent.rs       # LLM agent runtime
│       ├── db.rs          # DuckDB wrapper, schema introspection
│       ├── routes.rs      # HTTP handlers
│       ├── skill.rs       # Skill loader (YAML + markdown parser)
│       ├── llm.rs         # OpenAI client
│       ├── config.rs      # Configuration
│       └── bin/           # seed-data, seed-ports generators
├── skills/                # Drop-in skill directories
│   ├── ais_positions/     # Reference AIS skill
│   └── world_ports/       # World ports skill
└── data/seed/             # Generated parquet files
```

## Implementation Status

| Phase | Scope | Status |
|-------|-------|--------|
| 1 | MapLibre map, ship markers, selection, track rendering, command bar | Done |
| 2 | DuckDB-WASM, polygon/radius draw tools, spatial queries, table view | Done |
| 3 | Rust backend, skill loader, server DuckDB, API endpoints, vite proxy | Done |
| 4 | Agent runtime, LLM integration, command bar wiring, render pipeline | Done |
| 5 | Server-side voice (Whisper/Piper), ad hoc SQL hardening, extensibility proof | Not started |

## License

Private — not yet licensed.
