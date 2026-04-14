SELECT port_id, port_name, country, lat, lon, size, facilities
FROM world_ports
WHERE lat BETWEEN $min_lat AND $max_lat
  AND lon BETWEEN $min_lon AND $max_lon
ORDER BY port_name
LIMIT $limit
