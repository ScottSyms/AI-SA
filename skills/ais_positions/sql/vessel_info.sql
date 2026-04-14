-- Get detailed info for a specific vessel
-- Parameters: $mmsi (required)
SELECT
  mmsi,
  name,
  vessel_type,
  COUNT(*) as position_count,
  AVG(speed) as avg_speed,
  MAX(speed) as max_speed,
  MIN(lat) as min_lat,
  MAX(lat) as max_lat,
  MIN(lon) as min_lon,
  MAX(lon) as max_lon,
  MIN(timestamp) as first_seen,
  MAX(timestamp) as last_seen
FROM ais_positions
WHERE mmsi = $mmsi
GROUP BY mmsi, name, vessel_type
LIMIT 1;
