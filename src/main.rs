use csv::ReaderBuilder;
use extended_isolation_forest::{Forest, ForestOptions};
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;

#[derive(Debug, Deserialize, Clone)]
struct TripRecord {
    #[serde(rename = "Start station")]
    start_station: String,
    #[serde(rename = "End station")]
    end_station: String,
    #[serde(rename = "Total duration")]
    total_duration: String,
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

fn main() -> Result<(), Box<dyn Error>> {
    println!("Loading CSV...");

    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_path("LondonBikeJourneyAug2023.csv")?;

    let mut vectors: Vec<[f64; 3]> = Vec::new();
    let mut station_map: HashMap<String, f64> = HashMap::new();
    let mut station_counter = 0.0;

    let deserialized: csv::DeserializeRecordsIter<File, TripRecord> = reader.deserialize();

    println!("Vectorizing features...");

    let mut rows = vec![];

    for result in deserialized {
        let row = result?;

        rows.push(row.clone());

        let start_station_code = *station_map.entry(row.start_station).or_insert_with(|| {
            let current = station_counter;
            station_counter += 1.0;
            current
        });

        let end_station_code = *station_map.entry(row.end_station).or_insert_with(|| {
            let current = station_counter;
            station_counter += 1.0;
            current
        });

        let duration_str = row.total_duration;
        let seconds = duration_to_seconds(&duration_str)?;

        vectors.push([start_station_code, end_station_code, seconds]);
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

    let threshold = 0.6;

    let mut anomalous: Vec<(usize, f64)> = vec![];

    println!("Scoring with {} testing records...", testing_data.len());

    for (i, row) in testing_data.iter().enumerate() {
        let score = forest.score(row);
        if score > threshold {
            anomalous.push((i, score));
        }
    }

    println!("Found {} anomalous records.", anomalous.len());

    anomalous.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    let display_count = 10;
    println!("Top {} anomalous records:", display_count);

    for (i, score) in anomalous.iter().take(10) {
        println!("Row {}: {}", i, score);
        println!("{:?}", rows[*i]);
    }

    Ok(())
}
