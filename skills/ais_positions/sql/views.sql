-- AIS Positions: View definitions
-- Registered on startup by the platform skill loader

CREATE OR REPLACE VIEW v_latest_positions AS
SELECT DISTINCT ON (mmsi)
  mmsi, name, lat, lon, speed, heading, vessel_type, timestamp
FROM ais_positions
ORDER BY mmsi, timestamp DESC;

CREATE OR REPLACE VIEW v_vessel_tracks AS
SELECT mmsi, name, lat, lon, speed, heading, vessel_type, timestamp
FROM ais_positions
ORDER BY mmsi, timestamp ASC;

CREATE OR REPLACE VIEW v_vessel_summary AS
SELECT
  mmsi,
  name,
  vessel_type,
  COUNT(*) as position_count,
  AVG(speed) as avg_speed,
  MAX(speed) as max_speed,
  MIN(timestamp) as first_seen,
  MAX(timestamp) as last_seen
FROM ais_positions
GROUP BY mmsi, name, vessel_type;
