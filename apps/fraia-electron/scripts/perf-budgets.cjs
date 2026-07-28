const os = require('node:os');

const TIERS = {
  compact_laptop: {
    idleTotalWorkingSetMb: 650,
    idleRendererWorkingSetMb: 300,
    idleCpuPercent: 2,
    domNodesDefaultModel: 2500,
    viewportCanvasCount: 2,
    frameMs10k: 33,
    frameMs50k: 50,
    rendererWorkingSetMb50k: 900,
  },
  standard_laptop: {
    idleTotalWorkingSetMb: 900,
    idleRendererWorkingSetMb: 450,
    idleCpuPercent: 2,
    domNodesDefaultModel: 2500,
    viewportCanvasCount: 2,
    frameMs10k: 16.7,
    frameMs50k: 33,
    rendererWorkingSetMb50k: 1200,
  },
  workstation: {
    idleTotalWorkingSetMb: 1300,
    idleRendererWorkingSetMb: 650,
    idleCpuPercent: 2,
    domNodesDefaultModel: 2500,
    viewportCanvasCount: 2,
    frameMs10k: 16.7,
    frameMs50k: 25,
    rendererWorkingSetMb50k: 2000,
  },
};

function hardwareFacts() {
  return {
    totalMemGb: os.totalmem() / 1024 / 1024 / 1024,
    cpuCount: os.cpus().length,
    arch: process.arch,
    platform: process.platform,
  };
}

function detectTier(facts = hardwareFacts(), env = process.env) {
  const override = env.FRAIA_PERF_TIER;
  if (override && TIERS[override]) return override;
  if (facts.totalMemGb <= 16 || facts.cpuCount <= 6) return 'compact_laptop';
  if (facts.totalMemGb > 48 && facts.cpuCount >= 12) return 'workstation';
  return 'standard_laptop';
}

function selectedBudget(env = process.env) {
  const facts = hardwareFacts();
  const tier = detectTier(facts, env);
  return {
    tier,
    hardware: facts,
    budgets: TIERS[tier],
    override: env.FRAIA_PERF_TIER || null,
  };
}

function frameBudgetForMembers(budget, memberCount) {
  return memberCount <= 10000 ? budget.budgets.frameMs10k : budget.budgets.frameMs50k;
}

function rendererWorkingSetBudgetForMembers(budget, memberCount) {
  return memberCount >= 50000 ? budget.budgets.rendererWorkingSetMb50k : budget.budgets.idleRendererWorkingSetMb;
}

function round(value, digits = 1) {
  return Number.isFinite(value) ? Number(value.toFixed(digits)) : null;
}

module.exports = {
  TIERS,
  selectedBudget,
  frameBudgetForMembers,
  rendererWorkingSetBudgetForMembers,
  round,
};
