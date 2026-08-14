const fs = require('node:fs');
const path = require('node:path');
const { createHash } = require('node:crypto');
const { recognizeOcr } = require('../../ocr-runtime.cjs');

async function main() {
  const imageBytes = fs.readFileSync(path.join(__dirname, 'ocr', 'scanned-plan.png'));
  const result = await recognizeOcr({
    sourceId: 'offline-network-fixture',
    sourceSha256: createHash('sha256').update(imageBytes).digest('hex'),
    pageNumber: 1,
    rotationDegrees: 0,
    sourceCoordinateSpace: 'pdf-user-space-points',
    crop: { x0: 0, y0: 0, x1: 1200, y1: 800 },
    rasterWidth: imageBytes.readUInt32BE(16),
    rasterHeight: imageBytes.readUInt32BE(20),
    rasterToSourceTransform: [1, 0, 0, 1, 0, 0],
    nativeTextUsable: false,
    imageBytes,
  });
  if (result.status !== 'completed'
    || !result.candidates.some(({ text }) => /PLAN/i.test(text))) {
    throw new Error(`Offline OCR fixture failed: ${JSON.stringify(result)}`);
  }
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
