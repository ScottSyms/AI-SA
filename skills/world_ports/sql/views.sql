-- SQL views for world_ports skill

CREATE OR REPLACE VIEW v_all_ports AS
SELECT * FROM world_ports
ORDER BY port_name;

CREATE OR REPLACE VIEW v_port_summary AS
SELECT
    country,
    size,
    COUNT(*) as port_count
FROM world_ports
GROUP BY country, size
ORDER BY country, size;
