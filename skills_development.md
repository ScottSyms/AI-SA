# Skills Development Guide

This repo uses skills to add domain behavior without changing the platform core.

## What a Skill Is

A skill is a small, self-contained package that describes:

- its data source
- its supported interactions
- its tools
- its domain prompt
- its SQL views
- its map layers

The platform loads the skill and handles execution, rendering, validation, and routing.

## Current Skill Layout

The existing pattern is:

- `skills/<skill_name>/skill.md`
- `skills/<skill_name>/sql/*.sql`

Examples in this repo:

- `skills/ais_positions/skill.md`
- `skills/world_ports/skill.md`
- `skills/speech_output/skill.md`

## Skill File Structure

Each skill file starts with YAML front matter:

```yaml
---
skill: my_skill
source_type: parquet
source_path: data/seed/my_data.parquet

interactions:
  - multi_select
  - radius_query

map_layers:
  - id: my_layer
    type: circle
---
```

Then include these sections:

- `## Source`
- `## Tools`
- `## Domain prompt`
- `## SQL views`

## How To Create a New Skill

1. Create a new folder under `skills/`.
2. Add a `skill.md` file with front matter and the required sections.
3. Add SQL files under `skills/<skill_name>/sql/`.
4. Define the views your SQL uses in `views.sql`.
5. Keep tool names, parameters, and SQL filenames aligned.
6. Register any map layers the frontend should render.

## Can I Modify the Responses?

Yes, within the skill boundary.

You can change how the assistant responds by editing the skill’s `## Domain prompt`.
That prompt controls tone, terminology, priorities, and what the model should emphasize.

You can also shape responses indirectly by:

- changing the SQL output columns
- changing the tool descriptions
- changing the supported interactions
- changing the map layers and result structure

What you should not do:

- hardcode platform behavior in the skill
- bypass the render envelope
- invent schema that does not exist

## Can I Create New Data Sources?

Yes.

A skill can introduce a new data source by declaring a new `source_type` and `source_path`, then writing SQL against that source.

Typical examples:

- `parquet` files in `data/seed/`
- additional parquet datasets for new domains
- other source types if the platform loader supports them

You should also add:

- SQL views that expose a stable schema
- tool definitions that query those views
- a domain prompt that explains the dataset

## SQL Rules

Keep SQL in files, not inline in application code.

Follow these rules:

- use registered views
- keep queries bounded with `LIMIT`
- prefer read-only queries
- validate before execution
- avoid assuming spatial extensions are available

## Good Skill Design

A good skill is:

- narrow in scope
- explicit about its source data
- clear about its tools
- careful about naming
- stable in its views

## Minimal Template

```markdown
---
skill: example_skill
source_type: parquet
source_path: data/seed/example.parquet

interactions:
  - multi_select
  - viewport_filter

map_layers:
  - id: example_points
    type: circle
---

## Source

Describe the dataset here.

## Tools

### get_example
Describe what the tool returns.

## Domain prompt

Describe how the assistant should talk about this domain.

## SQL views

- `v_example_items`
- `v_example_summary`
```

## Practical Advice

- Start from `ais_positions` if you want a full reference.
- Use `world_ports` if you want a second, simpler example.
- Keep response wording inside the skill prompt, not in platform code.
- Add new data sources by adding new skills, not by expanding unrelated ones.

## Summary

Skills are the extension point.

- Yes, you can modify responses through the skill prompt and tool design.
- Yes, you can create new data sources by defining a new skill with its own source and SQL views.
