#!/usr/bin/env node
import fs from "fs";
import path from "path";

const REQUIRED_OPS = [
  "create_hunt",
  "add_clue",
  "submit_answer",
  "register_player",
  "get_leaderboard",
  "distribute_rewards",
  "mint_nft"
];

function readJson(filePath) {
  return JSON.parse(fs.readFileSync(filePath, "utf8"));
}

function fmtDeltaPct(current, baseline) {
  if (baseline === 0) {
    return current === 0 ? "0.00%" : "inf";
  }
  const pct = ((current - baseline) / baseline) * 100;
  return `${pct.toFixed(2)}%`;
}

function main() {
  const root = process.cwd();
  const baselinePath =
    process.argv[2] || path.join(root, "benchmarks", "gas-baseline.json");
  const currentPath =
    process.argv[3] || path.join(root, "benchmarks", "gas-current.json");

  if (!fs.existsSync(baselinePath)) {
    console.error(`Missing baseline file: ${baselinePath}`);
    process.exit(2);
  }
  if (!fs.existsSync(currentPath)) {
    console.error(`Missing current file: ${currentPath}`);
    process.exit(2);
  }

  const baseline = readJson(baselinePath);
  const current = readJson(currentPath);

  const tolerancePercent = Number(
    current.tolerance_percent ?? baseline.tolerance_percent ?? 10
  );
  const toleranceFactor = 1 + tolerancePercent / 100;

  let hardFailures = 0;
  let compared = 0;
  let skipped = 0;

  console.log(`Gas benchmark comparison (tolerance: ${tolerancePercent}%)`);

  for (const op of REQUIRED_OPS) {
    const b = baseline.operations?.[op]?.gas;
    const c = current.operations?.[op]?.gas;

    if (typeof c !== "number") {
      console.error(`- ${op}: missing or non-numeric current gas value`);
      hardFailures += 1;
      continue;
    }

    if (typeof b !== "number") {
      console.log(`- ${op}: skipped (baseline unset), current=${c}`);
      skipped += 1;
      continue;
    }

    compared += 1;

    if (b === 0) {
      if (c > 0) {
        console.error(`- ${op}: FAIL baseline=0 current=${c} delta=inf`);
        hardFailures += 1;
      } else {
        console.log(`- ${op}: OK baseline=0 current=0 delta=0.00%`);
      }
      continue;
    }

    const limit = Math.floor(b * toleranceFactor);
    const deltaPct = fmtDeltaPct(c, b);
    if (c > limit) {
      console.error(
        `- ${op}: FAIL baseline=${b} current=${c} limit=${limit} delta=${deltaPct}`
      );
      hardFailures += 1;
    } else {
      console.log(
        `- ${op}: OK baseline=${b} current=${c} limit=${limit} delta=${deltaPct}`
      );
    }
  }

  console.log(
    `Compared=${compared}, Skipped=${skipped}, Failures=${hardFailures}`
  );

  if (hardFailures > 0) {
    process.exit(1);
  }
}

main();
