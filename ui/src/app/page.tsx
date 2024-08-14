import { Map } from "@/components/Map";
import { journeysSchema } from "@/journey";
import { promises as fs } from "fs";

const repoPath =
  "/Users/luke.bennett/workspace/mdrx/internal/research/bike-share-anomaly-detection";
const dataPath = `${repoPath}/data/output.json`;

export default async function Home() {
  const journeys = journeysSchema.parse(
    JSON.parse(await fs.readFile(dataPath, "utf-8")),
  );

  return (
    <main>
      <Map journeys={journeys.slice(0, 10000)} />
    </main>
  );
}
