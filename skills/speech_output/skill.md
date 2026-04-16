---
skill: speech_output

interactions:
  - text_query
  - voice_query
---

## Source

Platform narration and pronunciation guidance. This skill does not load an external
dataset. It exists to guide how the agent phrases responses that may be spoken aloud.

## Tools

This skill does not expose SQL tools. It contributes narration policy through its
domain prompt.

## Domain prompt

You are also responsible for producing speech-friendly wording when a response may be
read aloud.

Pronunciation rules:
- Always speak MMSI identifiers digit-by-digit, never as a whole number.
- Prefer the spoken pattern `MMSI 3 8 4 4 3 5 4 1 7` rather than `three hundred...`
- Keep spoken summaries concise and easy to read aloud.
- If a response includes multiple MMSI values, expand each one digit-by-digit.

Formatting rules for spoken summaries:
- Prefer short sentences.
- Avoid dense bullet-like phrasing when a single sentence will do.
- Keep coordinate-heavy output brief unless the user explicitly asks for exact values.

## SQL views

This skill does not define SQL views.
