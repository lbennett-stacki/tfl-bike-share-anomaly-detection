use csv::ReaderBuilder;
use extended_isolation_forest::{Forest, ForestOptions};
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs::{read_to_string, File};
use std::io::prelude::*;

type Coords = (f64, f64);

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Journey {
    #[serde(rename(deserialize = "Start station", serialize = "startStation"))]
    start_station: String,

    #[serde(rename(deserialize = "Start coords", serialize = "startCoords"))]
    start_coords: Option<Coords>,

    #[serde(rename(deserialize = "End station", serialize = "endStation"))]
    end_station: String,

    #[serde(rename(deserialize = "End coords", serialize = "endCoords"))]
    end_coords: Option<Coords>,

    #[serde(rename(deserialize = "Total duration", serialize = "totalDuration"))]
    total_duration: String,

    #[serde(rename(deserialize = "Duration seconds", serialize = "durationSeconds"))]
    duration_seconds: Option<f64>,

    #[serde(rename(deserialize = "Score", serialize = "score"))]
    score: Option<f64>,
}

fn duration_to_seconds(duration: &str) -> Result<f64, String> {
    let mut total_seconds = 0.0;

    for part in duration.split_whitespace() {
        if let Some(days) = part.strip_suffix("d") {
            match days.parse::<f64>() {
                Ok(d) => total_seconds += d * 86400.0,
                Err(_) => return Err(format!("Invalid day format: {}", part)),
            }
        } else if let Some(hours) = part.strip_suffix("h") {
            match hours.parse::<f64>() {
                Ok(h) => total_seconds += h * 3600.0,
                Err(_) => return Err(format!("Invalid hour format: {}", part)),
            }
        } else if let Some(minutes) = part.strip_suffix("m") {
            match minutes.parse::<f64>() {
                Ok(m) => total_seconds += m * 60.0,
                Err(_) => return Err(format!("Invalid minute format: {}", part)),
            }
        } else if let Some(seconds) = part.strip_suffix("s") {
            match seconds.parse::<f64>() {
                Ok(s) => total_seconds += s,
                Err(_) => return Err(format!("Invalid second format: {}", part)),
            }
        } else {
            return Err(format!("Unknown duration part: {}", part));
        }
    }

    Ok(total_seconds)
}

#[derive(Deserialize)]
struct MapBoxGeometry {
    coordinates: [f64; 2],
}

#[derive(Deserialize)]
struct MapBoxFeature {
    geometry: MapBoxGeometry,
}

#[derive(Deserialize)]
struct MapBoxResponse {
    features: Vec<MapBoxFeature>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct DockingStation {
    name: String,
    lat: f64,
    long: f64,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct DockingStations {
    #[serde(rename = "$value")]
    stations: Vec<DockingStation>,
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("Loading CSV...");

    let repo_path =
        "/Users/luke.bennett/workspace/mdrx/internal/research/bike-share-anomaly-detection";

    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_path(format!("{}/data/input.csv", repo_path))?;

    let mut vectors: Vec<[f64; 2 + 2 + 1]> = Vec::new();

    let mut station_map: HashMap<String, Coords> = HashMap::new();

    let station_map_reader = File::open(format!("{}/data/station-map.json", repo_path));

    if station_map_reader.is_ok() {
        println!("Loading station map cache...");
        let raw = read_to_string(format!("{}/data/station-map.json", repo_path)).unwrap();
        station_map = serde_json::from_str(&raw)?;
    }

    let deserialized: csv::DeserializeRecordsIter<File, Journey> = reader.deserialize();

    let mut rows = vec![];

    let mut shuffled: Vec<_> = deserialized.collect();

    println!("Shuffling rows...");

    shuffled.shuffle(&mut thread_rng());

    println!("Vectorizing features...");

    let stations_raw = reqwest::blocking::get(
        "https://tfl.gov.uk/tfl/syndication/feeds/cycle-hire/livecyclehireupdates.xml",
    )
    .expect("Failed to fetch XML")
    .text()
    .expect("Failed to parse XML");
    let stations: DockingStations = serde_xml_rs::from_str(&stations_raw)?;

    for result in shuffled {
        let mut row = result?;

        let row_read = row.clone();

        let access_token = "";

        let start_station_coords = *station_map
            .entry(row_read.clone().start_station)
            .or_insert_with(|| {
                let station_from_xml = stations
                    .stations
                    .iter()
                    .find(|s| s.name == row_read.start_station);

                if let Some(station) = station_from_xml {
                    return (station.long, station.lat);
                }

                let query = format!("{}, London, UK", row_read.start_station);
                println!("Geocoding start: {}", query);
                let result = reqwest::blocking::get(format!(
                    "https://api.mapbox.com/geocoding/v5/mapbox.places/{}.json?access_token={}",
                    query, access_token
                ))
                .expect("Failed to geocode")
                .json::<MapBoxResponse>()
                .expect("Failed to parse geocode");

                let coords = &result.features[0].geometry.coordinates;
                (coords[0], coords[1])
            });

        row.start_coords = Some(start_station_coords);

        let end_station_coords = *station_map
            .entry(row_read.clone().end_station)
            .or_insert_with(|| {
                let station_from_xml = stations
                    .stations
                    .iter()
                    .find(|s| s.name == row_read.end_station);

                if let Some(station) = station_from_xml {
                    return (station.long, station.lat);
                }

                let query = format!("{}, London, UK", row_read.end_station);
                println!("Geocoding end: {}", query);
                let result = reqwest::blocking::get(format!(
                    "https://api.mapbox.com/geocoding/v5/mapbox.places/{}.json?access_token={}",
                    query, access_token
                ))
                .expect("Failed to geocode")
                .json::<MapBoxResponse>()
                .expect("Failed to parse geocode");

                let coords = &result.features[0].geometry.coordinates;
                (coords[0], coords[1])
            });

        row.end_coords = Some(end_station_coords);

        let duration_str = row_read.total_duration;
        let seconds = duration_to_seconds(&duration_str)?;
        row.duration_seconds = Some(seconds);

        rows.push(row.clone());

        vectors.push([
            start_station_coords.0,
            start_station_coords.1,
            end_station_coords.0,
            end_station_coords.1,
            seconds,
        ]);
    }

    println!("Vectorized {} records.", vectors.len());

    println!("Saving station map cache...");
    let mut station_map_writer = File::create(format!("{}/data/station-map.json", repo_path))?;
    let station_map_raw = serde_json::to_string_pretty(&station_map);
    match station_map_raw {
        Ok(raw) => {
            let _ = station_map_writer.write_all(raw.as_bytes());
        }
        Err(e) => {
            println!("Failed to serialize station map: {:?}", e);
        }
    }

    let options = ForestOptions {
        n_trees: 150,
        sample_size: 200,
        max_tree_depth: None,
        extension_level: 1,
    };

    if vectors.len() < 68_000 {
        println!("Not enough data to build the model");
        return Ok(());
    }

    let testing_data = vectors.clone()[0..72_000].to_vec();
    let training_data = vectors.clone()[72_000..].to_vec();

    println!(
        "Building isolation forest with {} training records...",
        training_data.len()
    );

    let forest = Forest::from_slice(training_data.as_slice(), &options).unwrap();

    let threshold = 0.76;

    let mut anomalous: Vec<(usize, f64)> = vec![];

    println!("Scoring with {} testing records...", testing_data.len());

    for (i, row) in testing_data.iter().enumerate() {
        let score = forest.score(row);
        rows[i].score = Some(score);
        if score > threshold {
            anomalous.push((i, score));
        }
    }

    println!("Found {} anomalous records.", anomalous.len());

    anomalous.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    let display_count = 10;
    println!("Top {} anomalous records:", display_count);

    for (i, score) in anomalous.iter().take(10) {
        println!("Row {}: {}", i, score);
        println!("{:?}", rows[*i]);
    }

    rows.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

    println!("Saving JSON output...");

    let mut writer = File::create(format!("{}/data/output.json", repo_path))?;
    let output_raw = serde_json::to_string(&rows);
    match output_raw {
        Ok(raw) => {
            let _ = writer.write_all(raw.as_bytes());
        }
        Err(e) => {
            println!("Failed to serialize output: {:?}", e);
        }
    }

    Ok(())
}
