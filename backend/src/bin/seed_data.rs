//! Generate synthetic AIS parquet seed data.
//!
//! Usage: cargo run --bin seed-data
//!
//! Produces data/seed/ais_sample.parquet with ~1000 vessels, each with ~50 track points spanning 90 days.

use duckdb::Connection;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::{HashMap, HashSet};
use std::fs;

const VESSEL_COUNT: usize = 1000;
const TRACK_POINTS: usize = 50;
const TRACK_SPAN_DAYS: i64 = 90;

const CARGO_NAMES: &[&str] = &[
    "Emma Maersk",
    "Ever Given",
    "MSC Gulsun",
    "Madrid Maersk",
    "CMA CGM Benjamin Franklin",
    "OOCL Hong Kong",
    "HMM Algeciras",
    "MOL Triumph",
    "CMA CGM Jacques Saade",
    "COSCO Shipping Nebula",
    "Ever Ace",
    "Ever Alot",
    "Ever Act",
    "Ever Forward",
    "MSC Oscar",
    "Maersk Eindhoven",
    "Maersk Essen",
    "Hapag-Lloyd Berlin",
    "ONE Integrity",
    "Hyundai Neptune",
    "Seaspan Bravo",
    "APL Atlanta",
    "CMA CGM Marco Polo",
    "OOCL Europe",
    "Ever Gifted",
];

const TANKER_NAMES: &[&str] = &[
    "TI Europe",
    "TI Asia",
    "Seawise Giant",
    "Front Altair",
    "Front Eagle",
    "Nave Andromeda",
    "Nave Ariadne",
    "DHT Tiger",
    "DHT Lion",
    "Almi Tankers",
    "Berge Everest",
    "Suezmax Trader",
    "Nordic Mistral",
    "Baltic Horizon",
    "Oceanic Pride",
    "Polar Endeavour",
    "Aframax Star",
    "Crested Falcon",
    "Ridgeway Spirit",
    "Meridian Voyager",
];

const FISHING_NAMES: &[&str] = &[
    "FV Cornelis Vrolijk",
    "FV Annelies Ilena",
    "FV Margiris",
    "FV Saga",
    "FV Atlantic Dawn",
    "FV Peterhead",
    "FV Westbank",
    "FV Ocean Harvest",
    "FV North Star",
    "FV Silver Dawn",
    "FV Sea Breeze",
    "FV Northern Quest",
    "FV Arctic Hunter",
    "FV Sapphire Tide",
    "FV Pacific Pride",
    "FV Blue Marlin",
    "FV Golden Sheaf",
    "FV Sea Hunter",
    "FV Harbour Light",
    "FV Ocean Venture",
];

const PASSENGER_NAMES: &[&str] = &[
    "Queen Mary 2",
    "Britannia",
    "Rotterdam",
    "AIDAcosma",
    "MSC World Europa",
    "Wonder of the Seas",
    "Icon of the Seas",
    "MS Europa",
    "MS Eurodam",
    "Costa Smeralda",
    "Carnival Vista",
    "Norwegian Encore",
    "Disney Dream",
    "Celebrity Edge",
    "Viking Venus",
    "Oasis of the Seas",
    "Quantum of the Seas",
    "Iona",
    "P&O Arvia",
    "Aurora",
];

const TUG_NAMES: &[&str] = &[
    "Svitzer Muir",
    "Svitzer Meridian",
    "Svitzer Ingrid",
    "Fairplay XI",
    "Fairplay 35",
    "Multratug 18",
    "Multratug 19",
    "Bourbon Orca",
    "Moran Explorer",
    "Viking Neptune",
    "Harbor Master",
    "Port Assist",
    "Dock Pilot",
    "Bay Tug",
    "Harbor Spirit",
];

const SAILING_NAMES: &[&str] = &[
    "Amerigo Vespucci",
    "Kruzenshtern",
    "Sedov",
    "Statsraad Lehmkuhl",
    "Sea Cloud",
    "Sea Cloud II",
    "Maltese Falcon",
    "Jadran",
    "Etoile du Roy",
    "Christian Radich",
    "Europa",
    "Royal Albatross",
    "Spirit of Bermuda",
    "Atyla",
    "Bluenose II",
];

const PLEASURE_NAMES: &[&str] = &[
    "A",
    "Eclipse",
    "Dilbar",
    "Azzam",
    "Rising Sun",
    "Koru",
    "Serene",
    "Octopus",
    "Lady Moura",
    "Flying Fox",
    "Nero",
    "Vava II",
    "Al Said",
    "Black Pearl",
    "Sailing Yacht A",
];

const MILITARY_NAMES: &[&str] = &[
    "USS Gerald R. Ford",
    "USS Zumwalt",
    "USS Arleigh Burke",
    "HMS Queen Elizabeth",
    "HMS Daring",
    "INS Vikrant",
    "JS Izumo",
    "FS Charles de Gaulle",
    "USS Nimitz",
    "HMS Prince of Wales",
    "INS Kolkata",
    "JS Kaga",
    "USS Monterey",
    "HMS Dragon",
    "FS Forbin",
];

const RESEARCH_NAMES: &[&str] = &[
    "RV Falkor",
    "RV Atlantis",
    "RV Polarstern",
    "RRS Sir David Attenborough",
    "NOAA Ship Okeanos Explorer",
    "RV Tara",
    "RV Neil Armstrong",
    "RV Investigator",
    "RV Metops",
    "RV Sonne",
    "RV Sikuliaq",
    "RV Maria S. Merian",
    "RV Revelle",
    "RV Pelagia",
    "RV Calypso",
];

const PILOT_NAMES: &[&str] = &[
    "Pilot 1",
    "Pilot 2",
    "Pilot 3",
    "Pilot 4",
    "Pilot 5",
    "Pilot 6",
    "Pilot 7",
    "Pilot 8",
    "Pilot 9",
    "Pilot 10",
    "Port Pilot",
    "Harbor Pilot",
    "Sea Pilot",
    "Dock Pilot",
    "Channel Pilot",
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

fn vessel_names(vessel_type: &str) -> &'static [&'static str] {
    match vessel_type {
        "Cargo" => CARGO_NAMES,
        "Tanker" => TANKER_NAMES,
        "Fishing" => FISHING_NAMES,
        "Passenger" => PASSENGER_NAMES,
        "Tug" => TUG_NAMES,
        "Sailing" => SAILING_NAMES,
        "Pleasure" => PLEASURE_NAMES,
        "Military" => MILITARY_NAMES,
        "Research" => RESEARCH_NAMES,
        "Pilot" => PILOT_NAMES,
        _ => CARGO_NAMES,
    }
}

fn pick_vessel_name(vessel_type: &str, index: usize, mmsi: i64) -> String {
    let names = vessel_names(vessel_type);
    if index < names.len() {
        names[index].to_string()
    } else {
        format!("{} {}", vessel_type, mmsi % 1000)
    }
}

fn main() {
    let project_root = resolve_project_root();

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
    let mut used_mmsi = HashSet::new();
    let mut vessel_name_counts: HashMap<&'static str, usize> = HashMap::new();

    let end_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time before UNIX_EPOCH")
        .as_secs() as i64;

    let mut insert_count = 0u64;

    for _ in 0..VESSEL_COUNT {
        let mmsi: i64 = loop {
            let candidate = 200_000_000 + rng.gen_range(0..600_000_000i64);
            if used_mmsi.insert(candidate) {
                break candidate;
            }
        };

        let region = pick_region(&mut rng);
        let base_lat = rng.gen_range(region.lat_min..region.lat_max);
        let base_lon = rng.gen_range(region.lon_min..region.lon_max);
        let base_speed: f64 = rng.gen_range(0.0..22.0);
        let base_heading: f64 = rng.gen_range(0.0..360.0);
        let vessel_type = VESSEL_TYPES[rng.gen_range(0..VESSEL_TYPES.len())];
        let name_index = vessel_name_counts.entry(vessel_type).or_insert(0);
        let name = pick_vessel_name(vessel_type, *name_index, mmsi);
        *name_index += 1;

        let mut lat = base_lat;
        let mut lon = base_lon;

        let start_time = end_time - TRACK_SPAN_DAYS * 24 * 60 * 60;

        let track_denominator = (TRACK_POINTS.saturating_sub(1)) as i64;

        for j in 0..TRACK_POINTS {
            let heading = base_heading + rng.gen_range(-15.0..15.0);
            let speed = (base_speed + rng.gen_range(-2.0..2.0)).max(0.0);
            let heading_int = ((heading % 360.0 + 360.0) % 360.0) as i32;
            let speed_rounded = (speed * 10.0).round() / 10.0;

            let ts_epoch = if track_denominator == 0 {
                start_time
            } else {
                start_time + ((TRACK_SPAN_DAYS * 24 * 60 * 60) * j as i64) / track_denominator
            };
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

fn resolve_project_root() -> std::path::PathBuf {
    if let Ok(root) = std::env::var("PROJECT_ROOT") {
        return std::path::PathBuf::from(root);
    }

    let cwd = std::env::current_dir().unwrap();
    if cwd.join("skills").is_dir() {
        cwd
    } else if cwd.join("../skills").is_dir() {
        cwd.join("..").canonicalize().unwrap_or_else(|_| cwd.join(".."))
    } else {
        cwd
    }
}
