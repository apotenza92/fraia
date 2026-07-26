const fs = require('node:fs');
const path = require('node:path');

const releaseRoot = path.resolve(__dirname, '..', 'release');
fs.rmSync(releaseRoot, { recursive: true, force: true });
fs.mkdirSync(releaseRoot, { recursive: true });
