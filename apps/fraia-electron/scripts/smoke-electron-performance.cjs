const { spawn } = require('node:child_process');
const fs = require('node:fs');
const path = require('node:path');
const { round, selectedBudget } = require('./perf-budgets.cjs');

function parseArgs(argv) {
  const config = {
    width: 1280,
    height: 800,
    delayMs: 3500,
    samples: 5,
    sampleIntervalMs: 1000,
    maxTotalWorkingSetMb: null,
    maxRendererWorkingSetMb: null,
    maxIdleCpuPercent: null,
    maxDomNodes: null,
    maxCanvasCount: null,
    output: path.join(process.cwd(), '..', '..', 'output', 'electron-performance-smoke.json'),
  };
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === '--width') config.width = Number(argv[++index] ?? config.width);
    else if (arg === '--height') config.height = Number(argv[++index] ?? config.height);
    else if (arg === '--delay-ms') config.delayMs = Number(argv[++index] ?? config.delayMs);
    else if (arg === '--samples') config.samples = Number(argv[++index] ?? config.samples);
    else if (arg === '--sample-interval-ms') config.sampleIntervalMs = Number(argv[++index] ?? config.sampleIntervalMs);
    else if (arg === '--max-total-working-set-mb') config.maxTotalWorkingSetMb = Number(argv[++index]);
    else if (arg === '--max-renderer-working-set-mb') config.maxRendererWorkingSetMb = Number(argv[++index]);
    else if (arg === '--max-idle-cpu-percent') config.maxIdleCpuPercent = Number(argv[++index]);
    else if (arg === '--max-dom-nodes') config.maxDomNodes = Number(argv[++index]);
    else if (arg === '--max-canvas-count') config.maxCanvasCount = Number(argv[++index]);
    else if (arg === '--output') config.output = path.resolve(argv[++index] ?? config.output);
    else if (arg === '--help' || arg === '-h') {
      console.log(`Fraia Electron performance smoke

Usage:
  npm run smoke:perf -- --max-total-working-set-mb 900

Options:
  --width <px>
  --height <px>
  --delay-ms <ms>
  --samples <count>
  --sample-interval-ms <ms>
  --max-total-working-set-mb <mb>
  --max-renderer-working-set-mb <mb>
  --max-idle-cpu-percent <percent>
  --max-dom-nodes <count>
  --max-canvas-count <count>
  --output <path>`);
      process.exit(0);
    } else {
      throw new Error(`Unknown argument: ${arg}`);
    }
  }
  return config;
}

function electronBin() {
  const resolved = require('electron');
  return typeof resolved === 'string' ? resolved : resolved.toString();
}

function fail(message) {
  console.error(message);
  process.exitCode = 1;
}

function metricForPid(metrics, pid) {
  return metrics.find((metric) => metric.pid === pid) ?? null;
}

async function runElectron(config) {
  fs.mkdirSync(path.dirname(config.output), { recursive: true });
  try {
    fs.rmSync(config.output, { force: true });
  } catch (_error) {
  }

  const env = {
    ...process.env,
    FRAIA_ELECTRON_METRICS_PATH: config.output,
    FRAIA_ELECTRON_CAPTURE_WIDTH: String(config.width),
    FRAIA_ELECTRON_CAPTURE_HEIGHT: String(config.height),
    FRAIA_ELECTRON_CAPTURE_DELAY_MS: String(config.delayMs),
    FRAIA_ELECTRON_METRICS_SAMPLES: String(config.samples),
    FRAIA_ELECTRON_METRICS_SAMPLE_INTERVAL_MS: String(config.sampleIntervalMs),
  };

  await new Promise((resolve, reject) => {
    const child = spawn(electronBin(), ['.'], {
      cwd: path.resolve(__dirname, '..'),
      env,
      stdio: 'inherit',
    });
    child.on('error', reject);
    child.on('exit', (code) => {
      if (code === 0) resolve();
      else reject(new Error(`Electron exited with code ${code}`));
    });
  });

  return JSON.parse(fs.readFileSync(config.output, 'utf8'));
}

async function main() {
  const config = parseArgs(process.argv.slice(2));
  const budget = selectedBudget();
  const effectiveBudgets = {
    maxTotalWorkingSetMb: config.maxTotalWorkingSetMb ?? budget.budgets.idleTotalWorkingSetMb,
    maxRendererWorkingSetMb: config.maxRendererWorkingSetMb ?? budget.budgets.idleRendererWorkingSetMb,
    maxIdleCpuPercent: config.maxIdleCpuPercent ?? budget.budgets.idleCpuPercent,
    maxDomNodes: config.maxDomNodes ?? budget.budgets.domNodesDefaultModel,
    maxCanvasCount: config.maxCanvasCount ?? budget.budgets.viewportCanvasCount,
  };
  const metrics = await runElectron(config);
  const rendererMetric = metricForPid(metrics.appMetrics ?? [], metrics.rendererPid);
  const totalWorkingSetMb = (metrics.appMetrics ?? [])
    .map((metric) => metric.workingSetMb)
    .filter(Number.isFinite)
    .reduce((sum, value) => sum + value, 0);
  const rendererWorkingSetMb = rendererMetric?.workingSetMb ?? null;
  const idleCpuPercent = metrics.idleSummary?.avgTotalCpuPercent ?? null;
  const domNodeCount = metrics.renderer?.domNodeCount ?? null;
  const canvasCount = metrics.renderer?.canvasCount ?? null;
  const canvasRoles = metrics.renderer?.canvasRoles ?? {};

  console.log(JSON.stringify({
    output: config.output,
    performanceBudget: metrics.performanceBudget ?? budget,
    idleCpuPercent: round(idleCpuPercent),
    maxIdleCpuPercent: round(metrics.idleSummary?.maxTotalCpuPercent),
    totalWorkingSetMb: round(totalWorkingSetMb),
    rendererWorkingSetMb: round(rendererWorkingSetMb),
    mainWorkingSetMb: round(metrics.mainProcessMemoryMb?.workingSetMb),
    domNodeCount,
    canvasCount,
    canvasRoles,
    viewport: metrics.renderer?.viewport,
  }, null, 2));

  if (Number.isFinite(effectiveBudgets.maxTotalWorkingSetMb) && totalWorkingSetMb > effectiveBudgets.maxTotalWorkingSetMb) {
    fail(`Total Electron working set ${round(totalWorkingSetMb)} MB exceeded budget ${effectiveBudgets.maxTotalWorkingSetMb} MB.`);
  }
  if (Number.isFinite(effectiveBudgets.maxRendererWorkingSetMb) && rendererWorkingSetMb > effectiveBudgets.maxRendererWorkingSetMb) {
    fail(`Renderer working set ${round(rendererWorkingSetMb)} MB exceeded budget ${effectiveBudgets.maxRendererWorkingSetMb} MB.`);
  }
  if (Number.isFinite(effectiveBudgets.maxIdleCpuPercent) && idleCpuPercent > effectiveBudgets.maxIdleCpuPercent) {
    fail(`Idle CPU ${round(idleCpuPercent)}% exceeded budget ${effectiveBudgets.maxIdleCpuPercent}%.`);
  }
  if (Number.isFinite(effectiveBudgets.maxDomNodes) && domNodeCount > effectiveBudgets.maxDomNodes) {
    fail(`DOM node count ${domNodeCount} exceeded budget ${effectiveBudgets.maxDomNodes}.`);
  }
  if (Number.isFinite(effectiveBudgets.maxCanvasCount) && canvasCount > effectiveBudgets.maxCanvasCount) {
    fail(`Canvas count ${canvasCount} exceeded budget ${effectiveBudgets.maxCanvasCount}.`);
  }
  if (canvasRoles['viewport-webgl'] !== 1 || canvasRoles['selection-overlay'] !== 1 || canvasRoles.unclassified) {
    fail(`Unexpected viewport canvas roles: ${JSON.stringify(canvasRoles)}.`);
  }

  if (process.exitCode) {
    process.exit(process.exitCode);
  }
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
