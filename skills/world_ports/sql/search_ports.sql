SELECT port_id, port_name, country, lat, lon, size, facilities
FROM world_ports
WHERE port_name ILIKE '%' || $search || '%'
   OR country ILIKE '%' || $search || '%'
ORDER BY port_name
LIMIT $limit
