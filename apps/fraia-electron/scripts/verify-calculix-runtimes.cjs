#!/usr/bin/env node

const path = require('node:path');
const { SUPPORTED_TARGETS } = require('../package-boundary.cjs');
const { validateRuntimeDirectory } = require('../calculix-runtime-manifest.cjs');

function main(argv = process.argv.slice(2)) {
  const all = argv.includes('--all');
  const skipDependencyInspection = argv.includes('--skip-dependency-inspection');
  const targets = all
    ? [...SUPPORTED_TARGETS]
    : [argv[argv.indexOf('--target') + 1] || `${process.platform}-${process.arch}`];
  for (const target of targets) {
    const directory = path.resolve(__dirname, '..', 'runtimes', 'calculix', target);
    validateRuntimeDirectory(directory, target, {
      inspectDependencies: !skipDependencyInspection,
    });
    process.stdout.write(`Verified reviewed CalculiX runtime: ${target}\n`);
  }
}

if (require.main === module) main();

module.exports = { main };
