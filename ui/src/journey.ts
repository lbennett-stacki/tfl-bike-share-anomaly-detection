import { z } from "zod";

export const coordsSchema = z.tuple([z.number(), z.number()]);

export const journeySchema = z.object({
  startStation: z.string(),
  startCoords: coordsSchema.nullable(),
  endStation: z.string(),
  endCoords: coordsSchema.nullable(),
  totalDuration: z.string(),
  durationSeconds: z.number(),
  score: z.number().nullable(),
});

export type Coords = [number, number];

export const journeysSchema = z.array(journeySchema);

export type Journey = z.infer<typeof journeySchema>;
