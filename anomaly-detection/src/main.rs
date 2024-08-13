use csv::ReaderBuilder;
use extended_isolation_forest::{Forest, ForestOptions};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;

type Coords = (f64, f64);

#[derive(Debug, Deserialize, Serialize, Clone)]
struct TripRecord {
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

fn main() -> Result<(), Box<dyn Error>> {
    println!("Loading CSV...");

    let repo_path =
        "/Users/luke.bennett/workspace/mdrx/internal/research/bike-share-anomaly-detection";

    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_path(format!("{}/data/input.csv", repo_path))?;

    let mut vectors: Vec<[f64; 2 + 2 + 1]> = Vec::new();

    let mut station_map: HashMap<String, Coords> = HashMap::new();

    let deserialized: csv::DeserializeRecordsIter<File, TripRecord> = reader.deserialize();

    println!("Vectorizing features...");

    let mut rows = vec![];

    for result in deserialized {
        let mut row = result?;

        let row_read = row.clone();

        let access_token = "MAPBOX_TOKEN";

        let start_station_coords = *station_map
            .entry(row_read.clone().start_station)
            .or_insert_with(|| {
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
                let coords = (coords[0], coords[1]);
                row.start_coords = Some(coords);

                coords
            });
        let end_station_coords = *station_map
            .entry(row_read.clone().end_station)
            .or_insert_with(|| {
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
                let coords = (coords[0], coords[1]);
                row.end_coords = Some(coords);

                coords
            });

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

    let options = ForestOptions {
        n_trees: 150,
        sample_size: 200,
        max_tree_depth: None,
        extension_level: 1,
    };

    if vectors.len() < 1000 {
        println!("Not enough data to build the model");
        return Ok(());
    }

    let testing_data = vectors.clone()[0..=100].to_vec();
    let training_data = vectors.clone()[100..].to_vec();

    println!(
        "Building isolation forest with {} training records...",
        training_data.len()
    );

    let forest = Forest::from_slice(training_data.as_slice(), &options).unwrap();

    let threshold = 0.5;

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

    println!("Saving JSON output...");

    let writer = File::create(format!("{}/data/output.json", repo_path))?;

    rows.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

    let _ = serde_json::to_writer(writer, &rows);

    Ok(())
}
