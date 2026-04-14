-- Get vessels within a bounding box
-- Parameters: $min_lon, $min_lat, $max_lon, $max_lat
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
WHERE lon BETWEEN $min_lon AND $max_lon
  AND lat BETWEEN $min_lat AND $max_lat
LIMIT 500;
