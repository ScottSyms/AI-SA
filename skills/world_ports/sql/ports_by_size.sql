SELECT port_id, port_name, country, lat, lon, size, facilities
FROM world_ports
WHERE size = $size
ORDER BY country, port_name
LIMIT $limit
