const path = require('node:path');
const {
  validateBuiltImportAssets,
  validateImportRuntimeSources,
} = require('../import-runtime-contract.cjs');

const appRoot = path.resolve(__dirname, '..');
const contract = validateImportRuntimeSources(appRoot);
const built = process.argv.includes('--built')
  ? validateBuiltImportAssets(appRoot)
  : null;

console.log(JSON.stringify({
  schema: contract.schema,
  targets: contract.targets,
  networkPolicy: contract.networkPolicy,
  ocr: contract.importers.ocr.behavior,
  built,
}));
