use std::{collections::HashMap, fs::read_to_string};

use serde::{Deserialize, Serialize};

type Coords = (f64, f64);

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Journey {
    #[serde(rename = "startStation")]
    start_station: String,

    #[serde(rename = "startCoords")]
    start_coords: Option<Coords>,

    #[serde(rename = "endStation")]
    end_station: String,

    #[serde(rename = "endCoords")]
    end_coords: Option<Coords>,

    #[serde(rename = "totalDuration")]
    total_duration: String,

    #[serde(rename = "durationSeconds")]
    duration_seconds: Option<f64>,

    #[serde(rename = "score")]
    score: Option<f64>,
}

fn main() {
    let repo_path =
        "/Users/luke.bennett/workspace/mdrx/internal/research/bike-share-anomaly-detection";

    println!("Loading JSON journeys...");
    let raw_journeys = read_to_string(format!("{}/data/output.json", repo_path)).unwrap();
    let journeys: Vec<Journey> = serde_json::from_str(&raw_journeys).unwrap();

    println!("Loaded {} journeys", journeys.len());

    let shortest_journey = journeys
        .iter()
        .min_by(|a, b| a.duration_seconds.partial_cmp(&b.duration_seconds).unwrap())
        .unwrap();

    let longest_journey = journeys
        .iter()
        .max_by(|a, b| a.duration_seconds.partial_cmp(&b.duration_seconds).unwrap())
        .unwrap();

    let most_common_start_station = journeys.iter().fold(HashMap::new(), |mut acc, journey| {
        *acc.entry(journey.start_station.clone()).or_insert(0) += 1;
        acc
    });
    let most_common_start_station = most_common_start_station
        .iter()
        .max_by(|a, b| a.1.cmp(b.1))
        .unwrap();

    let most_common_end_station = journeys.iter().fold(HashMap::new(), |mut acc, journey| {
        *acc.entry(journey.end_station.clone()).or_insert(0) += 1;
        acc
    });
    let most_common_end_station = most_common_end_station
        .iter()
        .max_by(|a, b| a.1.cmp(b.1))
        .unwrap();

    let least_common_start_station = journeys.iter().fold(HashMap::new(), |mut acc, journey| {
        *acc.entry(journey.start_station.clone()).or_insert(0) += 1;
        acc
    });
    let least_common_start_station = least_common_start_station
        .iter()
        .min_by(|a, b| a.1.cmp(b.1))
        .unwrap();

    let least_common_end_station = journeys.iter().fold(HashMap::new(), |mut acc, journey| {
        *acc.entry(journey.end_station.clone()).or_insert(0) += 1;
        acc
    });
    let least_common_end_station = least_common_end_station
        .iter()
        .min_by(|a, b| a.1.cmp(b.1))
        .unwrap();

    println!(
        "\nShortest journey is {:?} seconds long.\n{:?}\n",
        shortest_journey.duration_seconds.unwrap(),
        shortest_journey
    );

    println!(
        "\nLongest journey is {:?} days long.\n{:?}\n",
        longest_journey.duration_seconds.unwrap() / 86400.0,
        longest_journey
    );

    println!(
        "\nMost common start station is {:?} with {:?} journeys starting there.\n",
        most_common_start_station.0, most_common_start_station.1
    );

    println!(
        "\nLeast common start station is {:?} with {:?} journeys starting there.\n",
        least_common_start_station.0, least_common_start_station.1
    );

    println!(
        "\nMost common end station is {:?} with {:?} journeys ending there.\n",
        most_common_end_station.0, most_common_end_station.1
    );

    println!(
        "\nLeast common end station is {:?} with {:?} journeys ending there.\n",
        least_common_end_station.0, least_common_end_station.1
    );
}
