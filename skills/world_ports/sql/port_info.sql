SELECT port_id, port_name, country, lat, lon, size, facilities
FROM world_ports
WHERE port_id = $port_id
LIMIT 1
