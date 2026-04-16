⸻

AGENTS.md

Skill-Driven Geospatial Intelligence Platform (MapLibre + DuckDB + Agent Runtime)

⸻

0. Core Thesis

This system is a platform, not an application.
	•	The platform owns:
	•	rendering (MapLibre, tables, panels)
	•	interaction (mouse, text, voice)
	•	agent orchestration
	•	tool execution
	•	state management
	•	safety + validation
	•	Skills own:
	•	data sources
	•	schemas
	•	SQL logic
	•	domain prompts
	•	tool contracts
	•	map layer definitions

The platform knows how to render and interact.
Skills define what exists and how to reason about it.

⸻

1. Interaction Model (Non-Negotiable)

All interaction modes are first-class and equivalent:

1. Mouse (Primary spatial interaction)
	•	click selection
	•	multi-select
	•	lasso
	•	radius
	•	polygon draw
	•	viewport filtering

2. Text (Deterministic + ad hoc control)
	•	command-style queries
	•	structured operations
	•	analytics queries

3. Voice (Same pipeline as text)
	•	speech → text → agent → tools → render
	•	optional TTS response

Voice is NOT a separate system.
It feeds the same agent pipeline as text.

⸻

2. Architecture (Separation of Concerns)

Platform Layer (Stable Core)

Responsibilities:
	•	Map rendering (MapLibre GL JS — required)
	•	Table rendering
	•	Detail panels
	•	Selection state
	•	Interaction tools (lasso, radius, etc.)
	•	Agent runtime
	•	Tool dispatcher
	•	SQL execution engine (DuckDB)
	•	Voice pipeline (Whisper + Piper)
	•	Output normalization
	•	Logging / observability

Skill Layer (Plug-in Domain Logic)

Each skill:
	•	defines data sources
	•	declares tools
	•	provides SQL
	•	contributes agent prompt
	•	registers map layers
	•	declares supported interactions

Skills are drop-in. No platform code changes.

⸻

3. Map Engine (Hard Requirement)

Use MapLibre GL JS for all geospatial rendering.

Responsibilities
	•	base map rendering
	•	GeoJSON layers
	•	vector overlays
	•	feature selection
	•	highlighting
	•	viewport control
	•	event handling

Important Constraint

MapLibre does NOT handle drawing natively.

You MUST:
	•	integrate a drawing extension (e.g. mapbox-gl-draw fork)
	•	OR implement custom draw controls

⸻

4. Rendering Contract (Critical)

All outputs must normalize to a single envelope:

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

Rendering Rules

Field	Effect
geometry	Render on MapLibre
columns + data	Render table
single entity	Show detail panel
render_as	Renderer selection


⸻

5. Skill System (Merged Model)

Each skill is a Markdown file with YAML front matter:

skill: ais_positions
source_type: parquet
source_path: data/seed/**/*.parquet

interactions:
  - multi_select
  - lasso_select
  - radius_query
  - attribute_filter

map_layers:
  - id: ship_positions
    type: circle
  - id: ship_track
    type: line

Required Sections

## Source
## Tools
## Domain prompt
## SQL views


⸻

6. Skill Boundary Rule (Strict)

From your original architecture  ￼:

Skills declare intent and contracts.
The platform handles execution.

Expanded:

Concern	Owner
SQL logic	Skill
Tool signature	Skill
Tool execution	Platform
Rendering	Platform
Map layers	Skill (definition), Platform (rendering)
Interaction mechanics	Platform
Domain reasoning	Skill prompt


⸻

7. SQL-First Execution Model

DuckDB is the primary execution engine.

SQL lives in:
	•	/skills/{skill}/sql/*.sql

Rules:
	•	no inline SQL in code (except glue)
	•	read-only by default
	•	schema-registered views only
	•	always bounded (LIMIT)
	•	validated before execution

⸻

8. Tool Execution Model

Three execution paths:

Type	Runtime
SQL tools	DuckDB
Python tools	orchestration
Rust tools	high-performance (stdin/stdout)

Key Design Insight

You explicitly want:

“tools sitting in SQL files” + “Rust for speed”

So:
	•	SQL = default execution layer
	•	Python = orchestration glue
	•	Rust = acceleration path

⸻

9. Agent Layer (Merged Behavior)

The agent is:
	•	stateless interpreter
	•	skill router
	•	query planner
	•	tool caller

Execution Flow
	1.	interpret intent
	2.	resolve context (selection, geometry)
	3.	choose skill
	4.	try deterministic method
	5.	fallback to ad hoc SQL
	6.	validate
	7.	execute
	8.	normalize output
	9.	return renderable result

⸻

10. Context Model (Critical Fix from Your Issue)

Selection must include full identity, not just count.

{
  "mode": "multi",
  "items": [
    { "mmsi": 316..., "name": "..." }
  ],
  "geometry": {...},
  "primaryMMSI": 316...
}

Agent must receive:
	•	full list (capped)
	•	names
	•	geometry

⸻

11. Interaction Tools (Platform-Owned)

These are NOT skill tools:

Tool	Purpose
select_vessels	explicit selection
select_by_attribute	filtering
select_in_radius	spatial
clear_selection	reset
get_selection_summary	aggregation

Skills only declare:

interactions:
  - radius_query

Platform enables UI + tool.

⸻

12. AIS Skill (Reference Implementation)

Use your existing AIS skill as canonical baseline  ￼.

It already correctly demonstrates:
	•	SQL view abstraction
	•	domain prompt separation
	•	tool contracts
	•	overlay mutation model
	•	conflict handling

Do NOT redesign it — extend from it.

⸻

13. Loose Coupling (Your Key Requirement)

The system achieves loose coupling through:

1. Skills
	•	no imports into core platform
	•	loaded dynamically

2. SQL abstraction
	•	no direct data access in agent
	•	only via registered views

3. Tool dispatcher
	•	routing based on signature
	•	no domain branching

4. Map layer registration
	•	frontend reads /skills
	•	no hardcoded layers

⸻

14. What the Platform Must Never Do

Reject these patterns:
	•	❌ hardcoding AIS logic in backend
	•	❌ embedding SQL in agent prompts outside skill
	•	❌ tying UI to a specific dataset
	•	❌ making voice a separate execution path
	•	❌ bypassing tool validation
	•	❌ letting LLM invent schema

⸻

15. What the Platform Must Guarantee
	•	consistent rendering
	•	deterministic execution where possible
	•	safe ad hoc querying
	•	inspectable execution trace
	•	identical behavior across input modes

⸻

16. Implementation Phases (Condensed)

Phase 1
	•	MapLibre map
	•	ship markers
	•	selection

Phase 2
	•	track rendering
	•	selection model

Phase 3
	•	voice pipeline

Phase 4
	•	agent + skill dispatch

Phase 5
	•	ad hoc SQL + extensibility proof

⸻

17. Definition of Done

User can:
	•	click a ship
	•	ask “what is this ship”
	•	see identity
	•	render track
	•	draw polygon → filter vessels
	•	ask by voice
	•	run ad hoc query safely
	•	add/remove skill with zero code changes

⸻

18. Final Instruction

Build a generic geospatial agent platform where:
	•	MapLibre handles all spatial rendering
	•	DuckDB handles all structured computation
	•	skills inject domain capability
	•	tools define execution
	•	SQL defines logic
	•	agent orchestrates
	•	user interacts via mouse, text, or voice interchangeably

Do not collapse layers.
Do not hardcode domains.
Optimize for inspectability over abstraction.

⸻

19. Implementation Decisions (As-Built)

This section documents the actual technology choices and architectural decisions made during implementation.

Tech Stack

| Layer | Choice | Notes |
|-------|--------|-------|
| Frontend | SvelteKit + TypeScript (Svelte 5, runes mode) | Package manager: pnpm |
| Backend | Rust (Axum) | Async, tower-based |
| Database | DuckDB — hybrid deployment | WASM in browser + bundled in Rust backend |
| LLM | OpenAI (gpt-4o-mini default) | Abstracted behind LlmClient trait; model configurable via OPENAI_MODEL env var |
| Map tiles | OSM raster (interim) | Protomaps (pmtiles) installed for future vector tiles |
| Seed data | Synthetic AIS generator | 200 vessels × 50 track points = 10,000 rows in parquet |

Project Structure

```
tommy3/
├── frontend/          # SvelteKit app (port 5174, proxies /api/* → backend)
│   └── src/lib/
│       ├── platform/  # types, api client, stores (map, selection, duckdb, render, conversation, assistant)
│       ├── components/ # Map, CommandBar, TableView, Sidebar
│       └── data/       # synthetic AIS generator
├── backend/           # Rust Axum server (port 3001)
│   └── src/           # main, config, db, skill, routes, state, llm, agent, speech, tts
├── skills/            # Drop-in skill directories
│   ├── ais_positions/ # Reference skill with skill.md + sql/*.sql
│   └── speech_output/ # Narration/pronunciation guidance skill (no dataset)
└── data/seed/         # Generated parquet files
```

Drawing Implementation

MapLibre does not handle drawing natively. Rather than using mapbox-gl-draw (installed but unused), custom draw controls were implemented directly on MapLibre using GeoJSON sources and layers for polygon and radius tools. This gives full control over the interaction model and avoids the mapbox-gl-draw compatibility issues.

DuckDB Spatial Constraints

DuckDB-WASM in the browser does NOT have the spatial extension. Server-side DuckDB (bundled via duckdb-rs) also lacks spatial extensions. All spatial queries use:
- Bounding-box SQL (`WHERE lat BETWEEN ... AND lon BETWEEN ...`)
- Euclidean approximation for proximity sorting (`ORDER BY pow(lat - target, 2) + pow(lon - target, 2)`)
- Haversine formula for accurate distance in km
- Client-side ray-casting for point-in-polygon after bounding-box pre-filter

Agent Runtime

- OpenAI function-calling format with tool-calling loop (max 8 rounds)
- Built-in `run_sql` tool for ad hoc queries alongside skill-declared tools
- System prompt includes: DB schema, SQL examples, common geographic coordinates, explicit "no spatial extensions" constraint
- Final-round fallback: on the last iteration, tools are omitted to force a text-only summary from gathered data (prevents "exceeded max rounds" errors)
- Skill domain prompts are injected into the system prompt automatically
- Tool results are normalized to RenderEnvelope with automatic geometry detection (lat/lon → GeoJSON)

Frontend Rendering Pipeline

- Agent results flow through `agentResultStore` (Svelte 5 runes)
- Assistant UI behavior is centralized in `platform/assistant/store.ts`; Sidebar and CommandBar both subscribe to the same input, voice, status, and submission state
- Conversation history is session-scoped in `platform/conversation/store.ts` and is passed back to the backend agent as request context
- CommandBar dispatches to the store; Map, TableView, and Sidebar subscribe
- TableView is a unified lower-pane results surface: it shows agent result data, multi-selection tables, or single-vessel detail depending on context
- Map has an `agent-overlay` GeoJSON source with circle/line/fill layers; auto-fits bounds to results
- The primary workspace is vertically split: MapLibre map in the top pane, structured output in the bottom pane, with Sidebar fixed on the left
- Single-vessel details no longer appear as a floating popup over the map; they render in the lower pane instead
- All outputs conform to the RenderEnvelope contract from §4

Speech Input and Output

- Voice input currently uses the browser Web Speech API (`SpeechRecognition` / `webkitSpeechRecognition`) rather than Whisper
- Spoken output uses the OpenAI TTS API in the backend, with browser `speechSynthesis` fallback in the frontend if the API request fails
- The `speech_output` skill injects narration and pronunciation guidance into the agent prompt
- A platform-owned speech formatting pipeline normalizes spoken text before TTS on both the backend and frontend fallback path
- MMSI values are expanded to digit-by-digit pronunciation before speech synthesis (e.g. `MMSI 384435417` -> `MMSI 3 8 4 4 3 5 4 1 7`)
- The formatting pipeline is extensible so additional pronunciation rules can be added without changing agent logic or skills
- Pressing `Space` stops active speech playback first; if nothing is currently speaking, the same shortcut toggles voice input when focus is not in a text field

Rust Edition and Compatibility

- `Cargo.toml` uses `edition = "2024"` (requires Rust 1.85+)
- Rust 2024 reserves `gen` as a keyword — use `r#gen()` for rand 0.8's `.gen()` method
- duckdb-rs: call `stmt.query([])` before reading column metadata; use `ValueRef` (not `Value`) for row access
- macOS deployment target warnings during DuckDB bundled compilation are cosmetic

Revised Implementation Phases

The original phases were reorganized during implementation:

| Phase | Scope | Status |
|-------|-------|--------|
| 1 | MapLibre map + ship markers + click/multi-select + track rendering + command bar stub | Complete |
| 2 | DuckDB-WASM + polygon/radius draw tools + spatial queries + table view + render envelope | Complete |
| 3 | Rust Axum backend + skill loader + server DuckDB + API endpoints + vite proxy | Complete |
| 4 | Agent runtime + LLM integration + command bar wiring + frontend render pipeline | Complete |
| 5 | Browser voice input + OpenAI TTS with browser fallback + ad hoc SQL safety + second skill extensibility proof | In progress |
