const { spawnSync } = require('node:child_process');
const path = require('node:path');
const {
  frameBudgetForMembers,
  rendererWorkingSetBudgetForMembers,
  selectedBudget,
} = require('./perf-budgets.cjs');

function electronBin() {
  const resolved = require('electron');
  return typeof resolved === 'string' ? resolved : resolved.toString();
}

function runBenchmark(args) {
  const command = electronBin();
  const fullArgs = ['scripts/benchmark-three-viewport.cjs', ...args];
  console.log(`\n$ electron ${fullArgs.join(' ')}`);
  const result = spawnSync(command, fullArgs, {
    cwd: path.resolve(__dirname, '..'),
    env: process.env,
    encoding: 'utf8',
    stdio: 'inherit',
  });
  return result.status ?? 1;
}

function benchmarkArgs({ benchmark, members, frames }) {
  const budget = selectedBudget();
  return [
    '--mode', 'batched',
    '--benchmark', benchmark,
    '--members', String(members),
    '--labels', 'off',
    '--frames', String(frames),
    '--warmup', '8',
    '--max-avg-render-ms', String(frameBudgetForMembers(budget, members)),
    '--max-renderer-working-set-mb', String(rendererWorkingSetBudgetForMembers(budget, members)),
  ];
}

function main() {
  const budget = selectedBudget();
  console.log(JSON.stringify({ performanceBudget: budget }, null, 2));
  const matrix = [
    { benchmark: 'random', members: 10000, frames: 80 },
    { benchmark: 'multi', members: 50000, frames: 60 },
    { benchmark: 'portal', members: 50000, frames: 60 },
    { benchmark: 'random', members: 100000, frames: 45 },
  ];
  const failures = [];
  for (const item of matrix) {
    const status = runBenchmark(benchmarkArgs(item));
    if (status !== 0) failures.push(`${item.benchmark} ${item.members}`);
  }
  if (failures.length) {
    console.error(`\nPerformance gate failed: ${failures.join(', ')}`);
    process.exit(1);
  }
  console.log('\nPerformance gate passed.');
}

main();
