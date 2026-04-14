-- Get track history for a specific vessel
-- Parameters: $mmsi (required), $limit (default 100)
SELECT
  mmsi,
  lat,
  lon,
  speed,
  heading,
  timestamp
FROM v_vessel_tracks
WHERE mmsi = $mmsi
ORDER BY timestamp ASC
LIMIT COALESCE($limit, 100);
