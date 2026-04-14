-- Get latest position for each vessel
-- Parameters: $vessel_type (optional), $limit (default 200)
SELECT
  mmsi,
  name,
  lat,
  lon,
  speed,
  heading,
  vessel_type,
  timestamp
FROM v_latest_positions
WHERE ($vessel_type IS NULL OR vessel_type = $vessel_type)
LIMIT COALESCE($limit, 200);
