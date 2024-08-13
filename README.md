# TfL Bike Share Anomaly Detection

Isolation forest anomaly detection on start/end location and duration for TfL bike share.

![demo](./demo.png)

## Usage

First, download the [original dataset](https://www.kaggle.com/datasets/kalacheva/london-bike-share-usage-dataset).

```bash
# Set up data dir

mkdir data
mv ~/Downloads/[ORIGINAL_DATASET].csv  data/input.csv

# Train, test the last N rows, output enriched rows

cd anomaly-detection
cargo run

# Run UI

cd ui
npm install
npm run dev
```
