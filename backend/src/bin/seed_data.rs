//! Generate synthetic AIS parquet seed data.
//!
//! Usage: cargo run --bin seed-data
//!
//! Produces data/seed/ais_sample.parquet with ~200 vessels, each with ~50 track points.

use duckdb::Connection;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::fs;

const VESSEL_COUNT: usize = 200;
const TRACK_POINTS: usize = 50;

const VESSEL_NAMES: &[&str] = &[
    "Atlantic Pioneer",
    "Pacific Voyager",
    "Northern Star",
    "Southern Cross",
    "Sea Wanderer",
    "Ocean Titan",
    "Coral Venture",
    "Arctic Explorer",
    "Storm Chaser",
    "Blue Horizon",
    "Golden Eagle",
    "Silver Wave",
    "Red Falcon",
    "Iron Maiden",
    "Crystal Bay",
    "Thunder Bay",
    "Wind Spirit",
    "Polar Bear",
    "Sun Dancer",
    "Moon Shadow",
    "Neptune's Pride",
    "Emerald Isle",
    "Diamond Star",
    "Ruby Queen",
    "Sapphire Dream",
    "Pearl Harbor",
    "Jade Emperor",
    "Amber Dawn",
    "Crimson Tide",
    "Azure Sky",
    "Ivory Coast",
    "Ebony Night",
    "Topaz Sun",
    "Opal Moon",
    "Garnet Fire",
    "Onyx Shadow",
    "Turquoise Wave",
    "Coral Reef",
    "Lotus Flower",
    "Dragon Wing",
    "Phoenix Rise",
    "Falcon Crest",
    "Eagle Eye",
    "Hawk Wind",
    "Osprey Flight",
    "Heron Bay",
    "Pelican Point",
    "Albatross Wing",
    "Marlin Strike",
    "Swordfish Run",
];

const VESSEL_TYPES: &[&str] = &[
    "Cargo",
    "Tanker",
    "Fishing",
    "Passenger",
    "Tug",
    "Sailing",
    "Pleasure",
    "Military",
    "Research",
    "Pilot",
];

struct Region {
    lat_min: f64,
    lat_max: f64,
    lon_min: f64,
    lon_max: f64,
    weight: f64,
}

const REGIONS: &[Region] = &[
    Region {
        lat_min: 49.5,
        lat_max: 51.0,
        lon_min: -2.0,
        lon_max: 2.0,
        weight: 0.25,
    },
    Region {
        lat_min: 51.0,
        lat_max: 56.0,
        lon_min: 1.0,
        lon_max: 7.0,
        weight: 0.20,
    },
    Region {
        lat_min: 54.0,
        lat_max: 59.0,
        lon_min: 10.0,
        lon_max: 20.0,
        weight: 0.15,
    },
    Region {
        lat_min: 36.0,
        lat_max: 43.0,
        lon_min: -1.0,
        lon_max: 10.0,
        weight: 0.15,
    },
    Region {
        lat_min: 36.0,
        lat_max: 42.0,
        lon_min: -76.0,
        lon_max: -70.0,
        weight: 0.15,
    },
    Region {
        lat_min: 0.5,
        lat_max: 2.0,
        lon_min: 103.0,
        lon_max: 105.0,
        weight: 0.10,
    },
];

fn pick_region(rng: &mut StdRng) -> &'static Region {
    let r: f64 = rng.r#gen();
    let mut cumulative = 0.0;
    for region in REGIONS {
        cumulative += region.weight;
        if r <= cumulative {
            return region;
        }
    }
    &REGIONS[0]
}

fn main() {
    // Resolve output path
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

    let parquet_path = seed_dir.join("ais_sample.parquet");
    let parquet_str = parquet_path.to_string_lossy().to_string();

    println!("Generating seed data: {}", parquet_str);

    // Use DuckDB to create a table and export to parquet
    let conn = Connection::open_in_memory().expect("failed to open DuckDB");

    conn.execute_batch(
        "CREATE TABLE ais_positions (
            mmsi BIGINT,
            name VARCHAR,
            lat DOUBLE,
            lon DOUBLE,
            speed DOUBLE,
            heading INTEGER,
            vessel_type VARCHAR,
            timestamp TIMESTAMP
        )",
    )
    .expect("failed to create table");

    let mut rng = StdRng::seed_from_u64(42);
    let mut used_mmsi = std::collections::HashSet::new();

    let base_time = 1700000000i64; // ~Nov 2023 epoch seconds

    let mut insert_count = 0u64;

    for i in 0..VESSEL_COUNT {
        let mmsi: i64 = loop {
            let candidate = 200_000_000 + rng.gen_range(0..600_000_000i64);
            if used_mmsi.insert(candidate) {
                break candidate;
            }
        };

        let name = if i < VESSEL_NAMES.len() {
            VESSEL_NAMES[i].to_string()
        } else {
            format!("Vessel-{}", mmsi)
        };

        let region = pick_region(&mut rng);
        let base_lat = rng.gen_range(region.lat_min..region.lat_max);
        let base_lon = rng.gen_range(region.lon_min..region.lon_max);
        let base_speed: f64 = rng.gen_range(0.0..22.0);
        let base_heading: f64 = rng.gen_range(0.0..360.0);
        let vessel_type = VESSEL_TYPES[rng.gen_range(0..VESSEL_TYPES.len())];

        let mut lat = base_lat;
        let mut lon = base_lon;

        for j in 0..TRACK_POINTS {
            let heading = base_heading + rng.gen_range(-15.0..15.0);
            let speed = (base_speed + rng.gen_range(-2.0..2.0)).max(0.0);
            let heading_int = ((heading % 360.0 + 360.0) % 360.0) as i32;
            let speed_rounded = (speed * 10.0).round() / 10.0;

            let ts_epoch = base_time - ((TRACK_POINTS - 1 - j) as i64) * 600;
            // Format as ISO timestamp string for DuckDB
            let ts_str = format!("TIMESTAMP '{}'", chrono_from_epoch(ts_epoch));

            let escaped_name = name.replace('\'', "''");
            let sql = format!(
                "INSERT INTO ais_positions VALUES ({mmsi}, '{escaped_name}', {lat}, {lon}, {speed_rounded}, {heading_int}, '{vessel_type}', {ts_str})"
            );

            conn.execute_batch(&sql).unwrap_or_else(|e| {
                panic!("insert failed: {e}\nSQL: {sql}");
            });
            insert_count += 1;

            // Move vessel
            let heading_rad = heading.to_radians();
            let step = 0.005 * speed;
            lat -= heading_rad.cos() * step;
            lon -= heading_rad.sin() * step;
        }
    }

    println!(
        "Inserted {} position records for {} vessels",
        insert_count, VESSEL_COUNT
    );

    // Export to parquet
    let export_sql = format!("COPY ais_positions TO '{}' (FORMAT PARQUET)", parquet_str);
    conn.execute_batch(&export_sql)
        .expect("failed to export parquet");

    println!("Parquet written to: {}", parquet_str);

    // Verify
    let verify_sql = format!(
        "SELECT COUNT(*) as cnt FROM read_parquet('{}')",
        parquet_str
    );
    let mut stmt = conn.prepare(&verify_sql).unwrap();
    let count: i64 = stmt.query_row([], |row| row.get(0)).unwrap();
    println!("Verification: {} rows in parquet file", count);
}

/// Convert epoch seconds to an ISO-ish timestamp string.
fn chrono_from_epoch(epoch_secs: i64) -> String {
    let secs = epoch_secs;
    // Simple conversion without chrono dependency
    // 2023-11-14 22:13:20 for epoch 1700000000
    let days_since_epoch = secs / 86400;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    // Convert days since 1970-01-01 to date
    // Simple algorithm for dates after 2000
    let mut remaining_days = days_since_epoch;
    let mut year = 1970i32;

    loop {
        let days_in_year = if is_leap(year) { 366 } else { 365 };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        year += 1;
    }

    let months_days = if is_leap(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 1u32;
    for &md in &months_days {
        if remaining_days < md {
            break;
        }
        remaining_days -= md;
        month += 1;
    }
    let day = remaining_days + 1;

    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        year, month, day, hours, minutes, seconds
    )
}

fn is_leap(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}
