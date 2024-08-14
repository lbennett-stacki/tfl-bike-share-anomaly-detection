"use client";

import "mapbox-gl/dist/mapbox-gl.css";
import mapboxgl from "mapbox-gl";
import { useCallback, useEffect, useRef } from "react";
import { Coords, Journey } from "@/journey";

const mapboxStyleUrl =
  "mapbox://styles/lukeeeebennett/clzsii7jl00dx01qobjgs7hi3";

mapboxgl.accessToken = "";

const londonCenter: Coords = [-0.1276, 51.5072];

const threshold = 0.76;

export function Map({ journeys }: { journeys: Journey[] }) {
  const mapContainer = useRef<HTMLDivElement | null>(null);
  const map = useRef<mapboxgl.Map | null>(null);

  console.log(`Rendering ${journeys.length} journeys...`);

  const explain = useCallback(
    (journey: Journey) => {
      const journeysStartingHereCount = journeys.filter(
        (j) => j.startStation === journey.startStation,
      ).length;

      const journeysEndingHereCount = journeys.filter(
        (j) => j.endStation === journey.endStation,
      ).length;

      const journeysStartingAndEndingHereCount = journeys.filter(
        (j) =>
          j.startStation === journey.startStation &&
          j.endStation === journey.endStation,
      ).length;

      const averageDurationOfJourneysStartingHere =
        journeys
          .filter((j) => j.startStation === journey.startStation)
          .reduce((total, j) => total + j.durationSeconds, 0) /
        journeysStartingHereCount;

      const averageDurationOfJourneysEndingHere =
        journeys
          .filter((j) => j.endStation === journey.endStation)
          .reduce((total, j) => total + j.durationSeconds, 0) /
        journeysEndingHereCount;

      const averageDurationOfJourneysStartingAndEndingHere =
        journeys
          .filter(
            (j) =>
              j.startStation === journey.startStation &&
              j.endStation === journey.endStation,
          )
          .reduce((total, j) => total + j.durationSeconds, 0) /
        journeysStartingAndEndingHereCount;

      return `
        <ol>
          <li>
            * This journey is 1 of ${journeysStartingAndEndingHereCount} journeys from ${journey.startStation} to ${journey.endStation} with an average duration of ${averageDurationOfJourneysStartingAndEndingHere / 60 / 60} hours.
          </li>
          <li>
            * There are ${journeysStartingHereCount} journeys starting at ${journey.startStation} with an average duration of ${averageDurationOfJourneysStartingHere / 60 / 60} hours.
          </li>
          <li>
            * There are ${journeysEndingHereCount} journeys ending at ${journey.endStation} with an average duration of ${averageDurationOfJourneysEndingHere / 60 / 60} hours.
          </li>
        </ol>
      `.trim();
    },
    [journeys],
  );

  useEffect(() => {
    if (map.current) return;
    if (!mapContainer.current) return;

    map.current = new mapboxgl.Map({
      container: mapContainer.current,
      style: mapboxStyleUrl,
      center: londonCenter,
      zoom: 10,
    });

    map.current.on("load", () => {
      if (!map.current) {
        return;
      }

      const features = journeys.reduce((output, journey) => {
        console.log("Adding journey...", journey);

        if (journey.startCoords) {
          output.push({
            type: "Feature",
            properties: {},
            geometry: {
              type: "Point",
              coordinates: journey.startCoords,
            },
          });
        }

        if (journey.endCoords) {
          output.push({
            type: "Feature",
            properties: {},
            geometry: {
              type: "Point",
              coordinates: journey.endCoords,
            },
          });
        }

        if (journey.startCoords && journey.endCoords) {
          output.push({
            type: "Feature",
            properties: {},
            geometry: {
              type: "LineString",
              coordinates: [journey.startCoords, journey.endCoords],
            },
          });
        }

        return output;
      }, [] as any[]);

      const geojson: any = {
        type: "FeatureCollection",
        features,
      };

      console.log("Adding source...", geojson);

      map.current.addSource("training", {
        type: "geojson",
        data: geojson,
        // data: "https://docs.mapbox.com/mapbox-gl-js/assets/earthquakes.geojson",
      });

      map.current.addLayer({
        id: "training-heat",
        type: "heatmap",
        source: "training",
        paint: {
          "heatmap-opacity": 0.25,
        },
      });

      map.current.addLayer({
        id: "training-points",
        type: "circle",
        source: "training",
        paint: {
          "circle-radius": 5,
          "circle-color": "blue",
          "circle-opacity": {
            type: "exponential",
            property: "score",
            default: 0.5,
            stops: [
              [0, 0.5],
              [1, 1],
            ],
          },
        },
      });

      // map.current.addLayer({
      //   id: "training-lines",
      //   type: "line",
      //   source: "training",
      //   paint: {
      //     "line-color": "red",
      //     "line-opacity": 0.25,
      //     "line-width": 2,
      //     "line-blur": 1,
      //   },
      // });

      journeys.forEach((journey) => {
        if (!map.current) {
          return;
        }

        if (
          journey.startCoords &&
          journey.endCoords &&
          journey.score &&
          journey.score > threshold
        ) {
          map.current.addLayer({
            id: `${journey.startStation}-${journey.endStation}-${journey.score}-${journey.durationSeconds}`,
            type: "line",
            source: {
              type: "geojson",
              data: {
                type: "Feature",
                geometry: {
                  type: "LineString",
                  coordinates: [journey.startCoords, journey.endCoords],
                },
                properties: {},
              },
            },
            layout: {
              "line-join": "round",
              "line-cap": "round",
            },
            paint: {
              "line-color": !journey.score
                ? "orange"
                : journey.score > threshold
                  ? "red"
                  : "green",
              "line-width": journey.score > threshold ? 2 : 1,
            },
          });

          if (journey.score > threshold) {
            new mapboxgl.Marker({
              color: journey.score > threshold ? "red" : "blue",
            })
              .setLngLat(journey.startCoords)
              .setPopup(
                new mapboxgl.Popup().setHTML(
                  `
Start Node
<p>Start: ${journey.startStation}</p>
<p>End: ${journey.endStation}</p>
<p>Duration: ${journey.totalDuration}</p>
<p>Score: ${journey.score}</p>
<p>Explanation:</p>
${explain(journey)}
                `.trim(),
                ),
              )
              .addTo(map.current);
          }
        }
      });
    });
  }, [journeys, explain]);

  return <div ref={mapContainer} style={{ width: "100%", height: "100vh" }} />;
}
