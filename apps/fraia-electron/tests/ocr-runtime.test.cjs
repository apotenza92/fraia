const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { createHash } = require('node:crypto');
const { spawnSync } = require('node:child_process');
const test = require('node:test');
const {
  DEFAULT_OCR_POLICY,
  OCR_MODEL_SHA256,
  recognizeOcr,
} = require('../ocr-runtime.cjs');

const sourceInput = {
  sourceId: 'source-scanned-plan',
  sourceSha256: 'a'.repeat(64),
  pageNumber: 2,
  rotationDegrees: 90,
  sourceCoordinateSpace: 'pdf-user-space-points',
  crop: { x0: 10, y0: 20, x1: 210, y1: 120 },
  rasterWidth: 400,
  rasterHeight: 200,
  rasterToSourceTransform: [0.5, 0, 0, 0.5, 10, 20],
  nativeTextUsable: false,
  imageBytes: Buffer.from([137, 80, 78, 71]),
};

function fakeWorker(result, delayMillis = 0) {
  let terminated = false;
  return {
    get terminated() { return terminated; },
    async setParameters() {},
    async recognize() {
      if (delayMillis) await new Promise((resolve) => setTimeout(resolve, delayMillis));
      if (terminated) throw new Error('worker terminated');
      return { data: result };
    },
    async terminate() { terminated = true; },
  };
}

test('OCR emits exact unconfirmed provenance-bearing text candidates', async () => {
  const worker = fakeWorker({
    blocks: [{ paragraphs: [{ lines: [{ words: [{
      text: 'GROUND FLOOR PLAN',
      confidence: 92,
      bbox: { x0: 20, y0: 40, x1: 220, y1: 70 },
    }] }] }] }],
  });
  const result = await recognizeOcr(sourceInput, { createWorker: async () => worker });
  assert.equal(result.status, 'completed');
  assert.equal(result.sourceSha256, sourceInput.sourceSha256);
  assert.equal(result.rotationDegrees, 90);
  assert.equal(result.modelSha256, OCR_MODEL_SHA256);
  assert.equal(result.confirmation, 'unconfirmed');
  assert.equal(result.requiresConfirmation, true);
  assert.deepEqual(result.candidates[0].sourceBox, { x0: 20, y0: 40, x1: 120, y1: 55 });
  assert.equal(result.candidates[0].confidence, 0.92);
  assert.equal(result.candidates[0].confirmation, 'unconfirmed');
  assert.equal(worker.terminated, true);
});

test('OCR refuses pages whose native PDF text is usable', async () => {
  const result = await recognizeOcr({ ...sourceInput, nativeTextUsable: true });
  assert.equal(result.status, 'unavailable');
  assert.match(result.diagnostics[0].message, /only when native PDF text is absent/);
  assert.deepEqual(result.candidates, []);
});

test('OCR bounds raster bytes, pixels, and inferred output', async () => {
  const oversized = await recognizeOcr({
    ...sourceInput,
    imageBytes: Buffer.alloc(DEFAULT_OCR_POLICY.maxImageBytes + 1),
  });
  assert.equal(oversized.status, 'unavailable');

  const worker = fakeWorker({
    blocks: [{ paragraphs: [{ lines: [{ words: [
      { text: 'PLAN', confidence: 70, bbox: { x0: 0, y0: 0, x1: 1, y1: 1 } },
      { text: 'SECTION', confidence: 60, bbox: { x0: 1, y0: 0, x1: 2, y1: 1 } },
    ] }] }] }],
  });
  const limited = await recognizeOcr(sourceInput, {
    createWorker: async () => worker,
    policy: { maxWords: 1 },
  });
  assert.equal(limited.status, 'failed');
  assert.equal(limited.diagnostics[0].code, 'ocr.output-limit');
  assert.deepEqual(limited.candidates, []);
});

test('OCR cancellation terminates the worker and publishes no candidates', async () => {
  const controller = new AbortController();
  const worker = fakeWorker({ blocks: [] }, 100);
  setTimeout(() => controller.abort(), 5);
  const result = await recognizeOcr(sourceInput, {
    createWorker: async () => worker,
    signal: controller.signal,
  });
  assert.equal(result.status, 'cancelled');
  assert.equal(result.diagnostics[0].code, 'ocr.cancelled');
  assert.deepEqual(result.candidates, []);
  assert.equal(worker.terminated, true);
});

test('OCR timeout terminates the worker and publishes no candidates', async () => {
  const worker = fakeWorker({ blocks: [] }, 100);
  const result = await recognizeOcr(sourceInput, {
    createWorker: async () => worker,
    policy: { maxMillis: 5 },
  });
  assert.equal(result.status, 'timed_out');
  assert.equal(result.diagnostics[0].code, 'ocr.timed-out');
  assert.deepEqual(result.candidates, []);
  assert.equal(worker.terminated, true);
});

test('quarter-turn OCR boxes compose back through the exact source transform', async () => {
  const worker = fakeWorker({
    blocks: [{ paragraphs: [{ lines: [{ words: [
      { text: 'ELEVATION', confidence: 95, bbox: { x0: 366, y0: 678, x1: 1091, y1: 789 } },
      { text: '6000', confidence: 95, bbox: { x0: 297, y0: 990, x1: 658, y1: 1102 } },
    ] }] }] }],
  });
  const result = await recognizeOcr({
    ...sourceInput,
    rasterWidth: 1667,
    rasterHeight: 2500,
    rasterToSourceTransform: [800 / 1667, 0, 0, 1200 / 2500, 0, 0],
    ocrRotationRadians: -Math.PI / 2,
  }, { createWorker: async () => worker });
  expectCloseBox(result.candidates[0].sourceBox, {
    x0: (1667 - 789) * 800 / 1667,
    y0: 366 * 1200 / 2500,
    x1: (1667 - 678) * 800 / 1667,
    y1: 1091 * 1200 / 2500,
  });
  expectCloseBox(result.candidates[1].sourceBox, {
    x0: (1667 - 1102) * 800 / 1667,
    y0: 297 * 1200 / 2500,
    x1: (1667 - 990) * 800 / 1667,
    y1: 658 * 1200 / 2500,
  });
});

function expectCloseBox(actual, expected) {
  for (const key of ['x0', 'y0', 'x1', 'y1']) {
    assert.ok(Math.abs(actual[key] - expected[key]) < 1e-9, `${key}: ${actual[key]} != ${expected[key]}`);
  }
}

test('offline OCR reads deterministic scanned, rotated, and mixed drawing fixtures', { timeout: 120_000 }, async () => {
  const fixtures = [
    ['scanned-plan.png', /PLAN/i],
    ['scanned-elevation.png', /ELEVATION/i],
    ['scanned-section.png', /SECTION|PORTAL/i],
    ['rotated-elevation.png', /ELEVATION/i],
    ['mixed-plan-section.png', /PLAN|SECTION|PORTAL/i],
  ];
  for (const [file, expected] of fixtures) {
    const imageBytes = fs.readFileSync(path.join(__dirname, 'fixtures', 'ocr', file));
    const rasterWidth = imageBytes.readUInt32BE(16);
    const rasterHeight = imageBytes.readUInt32BE(20);
    const result = await recognizeOcr({
      ...sourceInput,
      sourceId: `fixture-${file}`,
      sourceSha256: createHash('sha256').update(imageBytes).digest('hex'),
      rasterWidth,
      rasterHeight,
      rasterToSourceTransform: [1, 0, 0, 1, 0, 0],
      ocrRotationRadians: file === 'rotated-elevation.png' ? -Math.PI / 2 : 0,
      imageBytes,
    });
    assert.equal(result.status, 'completed', `${file}: ${JSON.stringify(result.diagnostics)}`);
    assert.match(result.candidates.map(({ text }) => text).join(' '), expected, file);
    assert.ok(result.candidates.every(({ confirmation }) => confirmation === 'unconfirmed'));
  }
});

test('offline OCR completes when HTTP, HTTPS, and fetch are fail-closed', { timeout: 120_000 }, () => {
  const blocker = path.join(__dirname, 'fixtures', 'block-network.cjs');
  const runner = path.join(__dirname, 'fixtures', 'run-offline-ocr.cjs');
  const result = spawnSync(process.execPath, [runner], {
    encoding: 'utf8',
    env: { ...process.env, NODE_OPTIONS: `--require=${blocker}` },
    timeout: 120_000,
  });
  assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
});
