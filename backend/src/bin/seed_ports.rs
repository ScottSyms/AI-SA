//! Generate synthetic world ports parquet seed data.
//!
//! Usage: cargo run --bin seed-ports
//!
//! Produces data/seed/world_ports.parquet with ~80 ports worldwide.

use duckdb::Connection;
use std::fs;

struct Port {
    name: &'static str,
    country: &'static str,
    lat: f64,
    lon: f64,
    size: &'static str,
    facilities: &'static str,
}

const PORTS: &[Port] = &[
    // Northern Europe
    Port {
        name: "Rotterdam",
        country: "Netherlands",
        lat: 51.9,
        lon: 4.5,
        size: "Large",
        facilities: "cargo,tanker,container,repair,drydock",
    },
    Port {
        name: "Antwerp",
        country: "Belgium",
        lat: 51.3,
        lon: 4.4,
        size: "Large",
        facilities: "cargo,tanker,container,repair",
    },
    Port {
        name: "Hamburg",
        country: "Germany",
        lat: 53.5,
        lon: 10.0,
        size: "Large",
        facilities: "cargo,container,repair,drydock",
    },
    Port {
        name: "Bremerhaven",
        country: "Germany",
        lat: 53.5,
        lon: 8.6,
        size: "Large",
        facilities: "cargo,container",
    },
    Port {
        name: "Felixstowe",
        country: "United Kingdom",
        lat: 51.96,
        lon: 1.35,
        size: "Large",
        facilities: "cargo,container",
    },
    Port {
        name: "Southampton",
        country: "United Kingdom",
        lat: 50.9,
        lon: -1.4,
        size: "Large",
        facilities: "cargo,container,passenger,repair",
    },
    Port {
        name: "London Gateway",
        country: "United Kingdom",
        lat: 51.5,
        lon: 0.5,
        size: "Large",
        facilities: "cargo,container",
    },
    Port {
        name: "Dover",
        country: "United Kingdom",
        lat: 51.1,
        lon: 1.3,
        size: "Medium",
        facilities: "passenger,cargo",
    },
    Port {
        name: "Amsterdam",
        country: "Netherlands",
        lat: 52.4,
        lon: 4.8,
        size: "Large",
        facilities: "cargo,tanker,container",
    },
    Port {
        name: "Gothenburg",
        country: "Sweden",
        lat: 57.7,
        lon: 11.9,
        size: "Large",
        facilities: "cargo,container,tanker,repair",
    },
    Port {
        name: "Copenhagen",
        country: "Denmark",
        lat: 55.7,
        lon: 12.6,
        size: "Medium",
        facilities: "cargo,container,passenger",
    },
    Port {
        name: "Oslo",
        country: "Norway",
        lat: 59.9,
        lon: 10.7,
        size: "Medium",
        facilities: "cargo,container,passenger",
    },
    Port {
        name: "Bergen",
        country: "Norway",
        lat: 60.4,
        lon: 5.3,
        size: "Medium",
        facilities: "cargo,fishing,passenger",
    },
    Port {
        name: "Helsinki",
        country: "Finland",
        lat: 60.2,
        lon: 25.0,
        size: "Medium",
        facilities: "cargo,container,passenger",
    },
    Port {
        name: "St Petersburg",
        country: "Russia",
        lat: 59.9,
        lon: 30.3,
        size: "Large",
        facilities: "cargo,container,tanker",
    },
    Port {
        name: "Gdansk",
        country: "Poland",
        lat: 54.4,
        lon: 18.7,
        size: "Large",
        facilities: "cargo,container,tanker",
    },
    Port {
        name: "Le Havre",
        country: "France",
        lat: 49.5,
        lon: 0.1,
        size: "Large",
        facilities: "cargo,container,tanker",
    },
    Port {
        name: "Dunkirk",
        country: "France",
        lat: 51.0,
        lon: 2.4,
        size: "Medium",
        facilities: "cargo,tanker",
    },
    Port {
        name: "Zeebrugge",
        country: "Belgium",
        lat: 51.3,
        lon: 3.2,
        size: "Medium",
        facilities: "cargo,container",
    },
    // Mediterranean
    Port {
        name: "Barcelona",
        country: "Spain",
        lat: 41.4,
        lon: 2.2,
        size: "Large",
        facilities: "cargo,container,passenger,repair",
    },
    Port {
        name: "Valencia",
        country: "Spain",
        lat: 39.5,
        lon: -0.3,
        size: "Large",
        facilities: "cargo,container",
    },
    Port {
        name: "Algeciras",
        country: "Spain",
        lat: 36.1,
        lon: -5.4,
        size: "Large",
        facilities: "cargo,container,tanker",
    },
    Port {
        name: "Marseille",
        country: "France",
        lat: 43.3,
        lon: 5.4,
        size: "Large",
        facilities: "cargo,container,tanker,passenger",
    },
    Port {
        name: "Genoa",
        country: "Italy",
        lat: 44.4,
        lon: 8.9,
        size: "Large",
        facilities: "cargo,container,passenger,repair",
    },
    Port {
        name: "Naples",
        country: "Italy",
        lat: 40.8,
        lon: 14.3,
        size: "Medium",
        facilities: "cargo,container,passenger",
    },
    Port {
        name: "Piraeus",
        country: "Greece",
        lat: 37.9,
        lon: 23.6,
        size: "Large",
        facilities: "cargo,container,passenger",
    },
    Port {
        name: "Istanbul",
        country: "Turkey",
        lat: 41.0,
        lon: 29.0,
        size: "Large",
        facilities: "cargo,container,tanker,repair",
    },
    Port {
        name: "Malta Freeport",
        country: "Malta",
        lat: 35.8,
        lon: 14.5,
        size: "Medium",
        facilities: "cargo,container",
    },
    Port {
        name: "Tangier Med",
        country: "Morocco",
        lat: 35.9,
        lon: -5.5,
        size: "Large",
        facilities: "cargo,container",
    },
    Port {
        name: "Port Said",
        country: "Egypt",
        lat: 31.3,
        lon: 32.3,
        size: "Large",
        facilities: "cargo,container,tanker",
    },
    // Middle East
    Port {
        name: "Jeddah",
        country: "Saudi Arabia",
        lat: 21.5,
        lon: 39.2,
        size: "Large",
        facilities: "cargo,container,tanker",
    },
    Port {
        name: "Dubai (Jebel Ali)",
        country: "UAE",
        lat: 25.0,
        lon: 55.1,
        size: "Large",
        facilities: "cargo,container,tanker,drydock,repair",
    },
    Port {
        name: "Abu Dhabi",
        country: "UAE",
        lat: 24.5,
        lon: 54.4,
        size: "Large",
        facilities: "cargo,container,tanker",
    },
    Port {
        name: "Muscat",
        country: "Oman",
        lat: 23.6,
        lon: 58.6,
        size: "Medium",
        facilities: "cargo,container",
    },
    // Asia
    Port {
        name: "Singapore",
        country: "Singapore",
        lat: 1.3,
        lon: 103.8,
        size: "Large",
        facilities: "cargo,container,tanker,repair,drydock",
    },
    Port {
        name: "Shanghai",
        country: "China",
        lat: 31.2,
        lon: 121.5,
        size: "Large",
        facilities: "cargo,container,tanker,repair",
    },
    Port {
        name: "Shenzhen",
        country: "China",
        lat: 22.5,
        lon: 114.1,
        size: "Large",
        facilities: "cargo,container",
    },
    Port {
        name: "Ningbo-Zhoushan",
        country: "China",
        lat: 29.9,
        lon: 121.9,
        size: "Large",
        facilities: "cargo,container,tanker",
    },
    Port {
        name: "Hong Kong",
        country: "China",
        lat: 22.3,
        lon: 114.2,
        size: "Large",
        facilities: "cargo,container,passenger,repair",
    },
    Port {
        name: "Busan",
        country: "South Korea",
        lat: 35.1,
        lon: 129.0,
        size: "Large",
        facilities: "cargo,container,repair",
    },
    Port {
        name: "Tokyo",
        country: "Japan",
        lat: 35.7,
        lon: 139.8,
        size: "Large",
        facilities: "cargo,container,passenger",
    },
    Port {
        name: "Yokohama",
        country: "Japan",
        lat: 35.4,
        lon: 139.7,
        size: "Large",
        facilities: "cargo,container,passenger,repair",
    },
    Port {
        name: "Kobe",
        country: "Japan",
        lat: 34.7,
        lon: 135.2,
        size: "Large",
        facilities: "cargo,container",
    },
    Port {
        name: "Kaohsiung",
        country: "Taiwan",
        lat: 22.6,
        lon: 120.3,
        size: "Large",
        facilities: "cargo,container,repair",
    },
    Port {
        name: "Port Klang",
        country: "Malaysia",
        lat: 3.0,
        lon: 101.4,
        size: "Large",
        facilities: "cargo,container",
    },
    Port {
        name: "Tanjung Pelepas",
        country: "Malaysia",
        lat: 1.4,
        lon: 103.5,
        size: "Large",
        facilities: "cargo,container",
    },
    Port {
        name: "Laem Chabang",
        country: "Thailand",
        lat: 13.1,
        lon: 100.9,
        size: "Large",
        facilities: "cargo,container",
    },
    Port {
        name: "Ho Chi Minh City",
        country: "Vietnam",
        lat: 10.8,
        lon: 106.7,
        size: "Medium",
        facilities: "cargo,container",
    },
    Port {
        name: "Mumbai (Nhava Sheva)",
        country: "India",
        lat: 18.9,
        lon: 73.0,
        size: "Large",
        facilities: "cargo,container,tanker",
    },
    Port {
        name: "Colombo",
        country: "Sri Lanka",
        lat: 6.9,
        lon: 79.9,
        size: "Large",
        facilities: "cargo,container",
    },
    // Africa
    Port {
        name: "Durban",
        country: "South Africa",
        lat: -29.9,
        lon: 31.0,
        size: "Large",
        facilities: "cargo,container,tanker,repair",
    },
    Port {
        name: "Cape Town",
        country: "South Africa",
        lat: -33.9,
        lon: 18.4,
        size: "Medium",
        facilities: "cargo,container,fishing,repair",
    },
    Port {
        name: "Mombasa",
        country: "Kenya",
        lat: -4.0,
        lon: 39.7,
        size: "Medium",
        facilities: "cargo,container",
    },
    Port {
        name: "Lagos (Apapa)",
        country: "Nigeria",
        lat: 6.4,
        lon: 3.4,
        size: "Large",
        facilities: "cargo,container,tanker",
    },
    Port {
        name: "Djibouti",
        country: "Djibouti",
        lat: 11.6,
        lon: 43.1,
        size: "Medium",
        facilities: "cargo,container",
    },
    // Americas
    Port {
        name: "New York/New Jersey",
        country: "United States",
        lat: 40.7,
        lon: -74.0,
        size: "Large",
        facilities: "cargo,container,tanker,passenger",
    },
    Port {
        name: "Los Angeles",
        country: "United States",
        lat: 33.7,
        lon: -118.3,
        size: "Large",
        facilities: "cargo,container,tanker",
    },
    Port {
        name: "Long Beach",
        country: "United States",
        lat: 33.8,
        lon: -118.2,
        size: "Large",
        facilities: "cargo,container",
    },
    Port {
        name: "Houston",
        country: "United States",
        lat: 29.7,
        lon: -95.0,
        size: "Large",
        facilities: "cargo,tanker,container",
    },
    Port {
        name: "Savannah",
        country: "United States",
        lat: 32.1,
        lon: -81.1,
        size: "Large",
        facilities: "cargo,container",
    },
    Port {
        name: "Miami",
        country: "United States",
        lat: 25.8,
        lon: -80.2,
        size: "Large",
        facilities: "cargo,container,passenger",
    },
    Port {
        name: "Norfolk",
        country: "United States",
        lat: 36.8,
        lon: -76.3,
        size: "Large",
        facilities: "cargo,container",
    },
    Port {
        name: "Charleston",
        country: "United States",
        lat: 32.8,
        lon: -79.9,
        size: "Large",
        facilities: "cargo,container",
    },
    Port {
        name: "Vancouver",
        country: "Canada",
        lat: 49.3,
        lon: -123.1,
        size: "Large",
        facilities: "cargo,container,tanker",
    },
    Port {
        name: "Montreal",
        country: "Canada",
        lat: 45.5,
        lon: -73.6,
        size: "Large",
        facilities: "cargo,container",
    },
    Port {
        name: "Halifax",
        country: "Canada",
        lat: 44.6,
        lon: -63.6,
        size: "Medium",
        facilities: "cargo,container",
    },
    Port {
        name: "Santos",
        country: "Brazil",
        lat: -23.9,
        lon: -46.3,
        size: "Large",
        facilities: "cargo,container,tanker",
    },
    Port {
        name: "Buenos Aires",
        country: "Argentina",
        lat: -34.6,
        lon: -58.4,
        size: "Large",
        facilities: "cargo,container",
    },
    Port {
        name: "Colon",
        country: "Panama",
        lat: 9.4,
        lon: -79.9,
        size: "Large",
        facilities: "cargo,container",
    },
    Port {
        name: "Balboa",
        country: "Panama",
        lat: 9.0,
        lon: -79.6,
        size: "Large",
        facilities: "cargo,container",
    },
    Port {
        name: "Kingston",
        country: "Jamaica",
        lat: 17.97,
        lon: -76.8,
        size: "Medium",
        facilities: "cargo,container",
    },
    Port {
        name: "Cartagena",
        country: "Colombia",
        lat: 10.4,
        lon: -75.5,
        size: "Medium",
        facilities: "cargo,container",
    },
    // Oceania
    Port {
        name: "Melbourne",
        country: "Australia",
        lat: -37.8,
        lon: 145.0,
        size: "Large",
        facilities: "cargo,container",
    },
    Port {
        name: "Sydney",
        country: "Australia",
        lat: -33.9,
        lon: 151.2,
        size: "Large",
        facilities: "cargo,container,passenger",
    },
    Port {
        name: "Brisbane",
        country: "Australia",
        lat: -27.4,
        lon: 153.1,
        size: "Medium",
        facilities: "cargo,container",
    },
    Port {
        name: "Fremantle",
        country: "Australia",
        lat: -32.1,
        lon: 115.7,
        size: "Medium",
        facilities: "cargo,container",
    },
    Port {
        name: "Auckland",
        country: "New Zealand",
        lat: -36.8,
        lon: 174.8,
        size: "Medium",
        facilities: "cargo,container,passenger",
    },
    // Small ports (for variety)
    Port {
        name: "Peterhead",
        country: "United Kingdom",
        lat: 57.5,
        lon: -1.8,
        size: "Small",
        facilities: "fishing",
    },
    Port {
        name: "Lerwick",
        country: "United Kingdom",
        lat: 60.2,
        lon: -1.1,
        size: "Small",
        facilities: "fishing,cargo",
    },
    Port {
        name: "Torshavn",
        country: "Faroe Islands",
        lat: 62.0,
        lon: -6.8,
        size: "Small",
        facilities: "fishing,passenger",
    },
    Port {
        name: "Reykjavik",
        country: "Iceland",
        lat: 64.1,
        lon: -21.9,
        size: "Small",
        facilities: "fishing,cargo,passenger",
    },
];

fn main() {
    let cwd = std::env::current_dir().unwrap();
    let project_root = if cwd.join("data").is_dir() {
        cwd.clone()
    } else if cwd.join("../data").is_dir() {
        cwd.join("..")
    } else {
        cwd.clone()
    };

    let seed_dir = project_root.join("data").join("seed");
    fs::create_dir_all(&seed_dir).expect("failed to create data/seed/");

    let parquet_path = seed_dir.join("world_ports.parquet");
    let parquet_str = parquet_path.to_string_lossy().to_string();

    println!("Generating port seed data: {}", parquet_str);

    let conn = Connection::open_in_memory().expect("failed to open DuckDB");

    conn.execute_batch(
        "CREATE TABLE world_ports (
            port_id INTEGER,
            port_name VARCHAR,
            country VARCHAR,
            lat DOUBLE,
            lon DOUBLE,
            size VARCHAR,
            facilities VARCHAR
        )",
    )
    .expect("failed to create table");

    for (i, port) in PORTS.iter().enumerate() {
        let escaped_name = port.name.replace('\'', "''");
        let sql = format!(
            "INSERT INTO world_ports VALUES ({}, '{}', '{}', {}, {}, '{}', '{}')",
            i + 1,
            escaped_name,
            port.country.replace('\'', "''"),
            port.lat,
            port.lon,
            port.size,
            port.facilities,
        );
        conn.execute_batch(&sql).unwrap_or_else(|e| {
            panic!("insert failed: {e}\nSQL: {sql}");
        });
    }

    println!("Inserted {} ports", PORTS.len());

    let export_sql = format!("COPY world_ports TO '{}' (FORMAT PARQUET)", parquet_str);
    conn.execute_batch(&export_sql)
        .expect("failed to export parquet");

    println!("Parquet written to: {}", parquet_str);

    let verify_sql = format!(
        "SELECT COUNT(*) as cnt FROM read_parquet('{}')",
        parquet_str
    );
    let mut stmt = conn.prepare(&verify_sql).unwrap();
    let count: i64 = stmt.query_row([], |row| row.get(0)).unwrap();
    println!("Verification: {} rows in parquet file", count);
}
