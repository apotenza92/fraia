const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

const root = path.resolve(__dirname, '..');
const mainSource = fs.readFileSync(path.join(root, 'main.js'), 'utf8');
const preloadSource = fs.readFileSync(path.join(root, 'preload.js'), 'utf8');

test('native source import keeps raw file paths out of the renderer bridge', () => {
  const main = fs.readFileSync(path.join(root, 'main.js'), 'utf8');
  const preload = fs.readFileSync(path.join(root, 'preload.js'), 'utf8');
  const globalTypes = fs.readFileSync(path.join(root, 'src', 'global.d.ts'), 'utf8');

  assert.match(main, /ipcMain\.handle\('fraia:importSource'[\s\S]*?dialog\.showOpenDialog/);
  assert.match(main, /callApi\('\/sources\/selections\/issue'[\s\S]*?selectedPath: result\.filePaths\[0\]/);
  assert.match(main, /callApi\('\/sources\/import'[\s\S]*?selectionToken: grant\.selectionToken/);
  assert.match(preload, /importSource: \(payload\) => ipcRenderer\.invoke\('fraia:importSource', \{ projectDir: payload\.projectDir \}\)/);
  assert.doesNotMatch(preload, /importSource:[^\n]*selectedPath/);
  assert.match(globalTypes, /importSource: \(request: \{ projectDir: string \}\)/);
  assert.doesNotMatch(globalTypes, /importSource:[^\n]*selectedPath/);
});

test('source import progress exposes states but never native path data', () => {
  const main = fs.readFileSync(path.join(root, 'main.js'), 'utf8');
  const preload = fs.readFileSync(path.join(root, 'preload.js'), 'utf8');
  assert.match(main, /sourceImportProgress', \{ state: 'uploading' \}/);
  assert.match(main, /sourceImportProgress', \{ state: 'processing' \}/);
  assert.match(main, /sourceImportProgress', \{ state: 'done' \}/);
  assert.doesNotMatch(main, /sourceImportProgress', \{[^}]*filePath/);
  assert.match(preload, /onSourceImportProgress/);
});

test('PDF rendering reads only verified managed bytes and exposes no project object path', () => {
  assert.match(mainSource, /ipcMain\.handle\('fraia:readPdfSource'/);
  assert.match(mainSource, /callApi\('\/sources\/inspect'/);
  assert.match(mainSource, /fs\.promises\.realpath\(sourcesDir\)/);
  assert.match(mainSource, /createHash\('sha256'\)\.update\(bytes\)\.digest\('hex'\) !== source\.sha256/);
  assert.match(mainSource, /return Uint8Array\.from\(bytes\)/);
  assert.match(preloadSource, /readPdfSource: \(payload\) => ipcRenderer\.invoke\('fraia:readPdfSource', \{ projectDir: payload\.projectDir, sourceId: payload\.sourceId \}\)/);
  assert.doesNotMatch(preloadSource, /object_path|objectPath|selectedPath/);
});

test('PDF.js is pinned for an offline worker and product code does not invoke Poppler', () => {
  const packageMetadata = JSON.parse(fs.readFileSync(path.join(root, 'package.json'), 'utf8'));
  const browserSource = fs.readFileSync(path.join(root, 'src/components/sources/PdfPageBrowser.tsx'), 'utf8');
  const productSources = [mainSource, preloadSource, browserSource].join('\n');
  assert.equal(packageMetadata.devDependencies['pdfjs-dist'], '6.2.108');
  assert.equal(packageMetadata.dependencies?.['pdfjs-dist'], undefined, 'bundled PDF.js must not ship as a runtime Node dependency');
  assert.match(browserSource, /pdf\.worker\.min\.mjs\?url/);
  assert.match(browserSource, /GlobalWorkerOptions\.workerSrc = pdfWorkerUrl/);
  assert.doesNotMatch(productSources, /pdftoppm|pdfinfo|poppler/i);
});

test('OCR stays in the main process and returns inferred candidates without native paths', () => {
  assert.match(mainSource, /ipcMain\.handle\('fraia:recognizePdfOcr'[\s\S]*?recognizeOcr/);
  assert.match(mainSource, /\[ocr\] Started[\s\S]*?raster=[\s\S]*?bytes=[\s\S]*?rotation=/);
  assert.match(mainSource, /\[ocr\] Finished status=[\s\S]*?candidates=[\s\S]*?elapsedMillis=/);
  assert.match(preloadSource, /recognizePdfOcr: \(payload\) => ipcRenderer\.invoke\('fraia:recognizePdfOcr', payload\)/);
  assert.doesNotMatch(preloadSource, /OCR_MODEL_PATH|traineddata|tesseract-core|ocr-runtime/);
  assert.match(fs.readFileSync(path.join(root, 'src/components/sources/PdfPageBrowser.tsx'), 'utf8'), /Reading scanned text…[\s\S]*?Choose the drawing view manually/);
});

test('drawing interpretation and canonical run readers expose only typed project-scoped routes', () => {
  const contracts = [
    ['listDrawingInterpretations', '/interpretations/list'],
    ['inspectDrawingInterpretation', '/interpretations/inspect'],
    ['createDrawingInterpretation', '/interpretations/create'],
    ['confirmDrawingObservations', '/interpretations/confirm'],
    ['correctDrawingObservation', '/interpretations/correct'],
    ['reconcileDrawingInterpretation', '/interpretations/reconcile'],
    ['resolveDrawingInterpretationConflict', '/interpretations/conflicts/resolve'],
    ['listDesignRuns', '/design-runs/list'],
    ['inspectDesignRun', '/design-runs/inspect'],
    ['listDesignRunStatuses', '/design-runs/status'],
  ];
  for (const [name, route] of contracts) {
    assert.match(preloadSource, new RegExp(`${name}: \\(payload\\) => ipcRenderer\\.invoke\\('fraia:${name}', payload\\)`));
    assert.match(mainSource, new RegExp(`ipcMain\\.handle\\('fraia:${name}'[\\s\\S]*?callApi\\('${route.replaceAll('/', '\\/')}'`));
  }
  assert.doesNotMatch(preloadSource, /interpretationsDir|runsDir|attachmentPath|nativePath/);
});

test('DXF indexing and selection preparation stay project scoped without native paths', () => {
  const contracts = [
    ['indexDxfSource', '/dxf/index'],
    ['prepareDxfSelection', '/dxf/selections/prepare'],
  ];
  for (const [name, route] of contracts) {
    assert.match(preloadSource, new RegExp(`${name}: \\(payload\\) => ipcRenderer\\.invoke\\('fraia:${name}', payload\\)`));
    assert.match(mainSource, new RegExp(`ipcMain\\.handle\\('fraia:${name}'[\\s\\S]*?callApi\\('${route.replaceAll('/', '\\/')}'`));
  }
  assert.doesNotMatch(preloadSource, /dxfOriginalPath|dxfNativePath|dxfSelectedPath/);
});

test('PDF inference and IFC selection stay project scoped without native paths', () => {
  const contracts = [
    ['inferPdfViewRole', '/pdf/view-role/infer'],
    ['indexIfcSource', '/ifc/index'],
    ['prepareIfcSelection', '/ifc/selections/prepare'],
  ];
  for (const [name, route] of contracts) {
    assert.match(preloadSource, new RegExp(`${name}: \\(payload\\) => ipcRenderer\\.invoke\\('fraia:${name}', payload\\)`));
    assert.match(mainSource, new RegExp(`ipcMain\\.handle\\('fraia:${name}'[\\s\\S]*?callApi\\('${route.replaceAll('/', '\\/')}'`));
  }
  assert.doesNotMatch(preloadSource, /ifcNativePath|pdfInferencePath/);
});

test('neutral mesh preview uses bounded managed bytes and opaque cancellable jobs', () => {
  for (const [name, route] of [
    ['startMeshIndex', '/meshes/jobs/start'],
    ['meshIndexStatus', '/meshes/jobs/status'],
    ['cancelMeshIndex', '/meshes/jobs/cancel'],
    ['prepareMeshSavedView', '/meshes/saved-views/prepare'],
  ]) {
    assert.match(preloadSource, new RegExp(`${name}: \\(payload\\) => ipcRenderer\\.invoke\\('fraia:${name}', payload\\)`));
    assert.match(mainSource, new RegExp(`ipcMain\\.handle\\('fraia:${name}'[\\s\\S]*?callApi\\('${route.replaceAll('/', '\\/')}'`));
  }
  assert.match(mainSource, /ipcMain\.handle\('fraia:readMeshContent'/);
  assert.match(mainSource, /x-fraia-source-sha256/);
  assert.match(mainSource, /response\.arrayBuffer\(\)/);
  assert.doesNotMatch(preloadSource, /meshNativePath|meshObjectPath|selectedPath/);
});

test('analysis attempts expose typed project-scoped start status and cancel only', () => {
  for (const [name, route] of [
    ['startAnalysisAttempt', '/analysis-attempts/start'],
    ['analysisAttemptStatus', '/analysis-attempts/status'],
    ['cancelAnalysisAttempt', '/analysis-attempts/cancel'],
  ]) {
    assert.match(preloadSource, new RegExp(`${name}: \\(payload\\) => ipcRenderer\\.invoke\\('fraia:${name}', payload\\)`));
    assert.match(mainSource, new RegExp(`ipcMain\\.handle\\('fraia:${name}'[\\s\\S]*?callApi\\('${route.replaceAll('/', '\\/')}'`));
  }
  assert.match(mainSource, /if \(!app\.isPackaged && \/\^\\d\+\$\/[\s\S]*?FRAIA_TEST_ANALYSIS_DELAY_MS/);
  assert.match(mainSource, /if \(!app\.isPackaged && process\.env\.FRAIA_TEST_ANALYSIS_FAILURE === '1'\)/);
  assert.doesNotMatch(preloadSource, /FRAIA_TEST_ANALYSIS_DELAY_MS|FRAIA_TEST_ANALYSIS_FAILURE/);
});
