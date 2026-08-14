const fs = require('node:fs');
const path = require('node:path');
const { createHash } = require('node:crypto');
const { createWorker, OEM, PSM } = require('tesseract.js');

const OCR_SCHEMA = 'fraia.pdf-ocr-candidates.v1';
const OCR_ENGINE = 'tesseract.js';
const OCR_ENGINE_VERSION = '7.0.0';
const OCR_LANGUAGE = 'eng';
const OCR_MODEL_REPOSITORY = 'tesseract-ocr/tessdata_fast';
const OCR_MODEL_COMMIT = '65727574dfcd264acbb0c3e07860e4e9e9b22185';
const OCR_MODEL_SHA256 = '7d4322bd2a7749724879683fc3912cb542f19906c83bcc1a52132556427170b2';
const OCR_MODEL_PATH = path.join(__dirname, 'ocr-runtime', 'eng.traineddata');

const DEFAULT_OCR_POLICY = Object.freeze({
  maxImageBytes: 32 * 1024 * 1024,
  maxPixels: 50_000_000,
  maxWords: 50_000,
  maxCharacters: 2_000_000,
  maxMillis: 120_000,
});

function diagnostic(code, message) {
  return { code, message };
}

function terminalResult(input, status, diagnostics, elapsedMillis, candidates = []) {
  return {
    schema: OCR_SCHEMA,
    status,
    sourceId: input.sourceId,
    sourceSha256: input.sourceSha256,
    pageNumber: input.pageNumber,
    rotationDegrees: input.rotationDegrees,
    ocrRotationRadians: input.ocrRotationRadians || 0,
    sourceCoordinateSpace: input.sourceCoordinateSpace,
    crop: input.crop,
    rasterWidth: input.rasterWidth,
    rasterHeight: input.rasterHeight,
    engine: OCR_ENGINE,
    engineVersion: OCR_ENGINE_VERSION,
    language: OCR_LANGUAGE,
    modelRepository: OCR_MODEL_REPOSITORY,
    modelCommit: OCR_MODEL_COMMIT,
    modelSha256: OCR_MODEL_SHA256,
    extractionMethod: 'ocr',
    confirmation: 'unconfirmed',
    requiresConfirmation: true,
    elapsedMillis,
    candidates,
    diagnostics,
  };
}

function finiteNumber(value) {
  return typeof value === 'number' && Number.isFinite(value);
}

function validateInput(input, policy) {
  const bytes = Buffer.from(input.imageBytes || []);
  if (!input.sourceId || !/^[a-f0-9]{64}$/.test(input.sourceSha256 || '')) {
    throw new Error('OCR source identity is missing or invalid.');
  }
  if (!Number.isInteger(input.pageNumber) || input.pageNumber < 1) {
    throw new Error('OCR page number must be a positive integer.');
  }
  if (!Number.isInteger(input.rasterWidth) || !Number.isInteger(input.rasterHeight)
    || input.rasterWidth < 1 || input.rasterHeight < 1) {
    throw new Error('OCR raster dimensions must be positive integers.');
  }
  if (bytes.length === 0 || bytes.length > policy.maxImageBytes) {
    throw new Error(`OCR raster bytes exceed the ${policy.maxImageBytes} byte limit.`);
  }
  if (input.rasterWidth * input.rasterHeight > policy.maxPixels) {
    throw new Error(`OCR raster pixels exceed the ${policy.maxPixels} pixel limit.`);
  }
  if (input.nativeTextUsable !== false) {
    throw new Error('OCR is allowed only when native PDF text is absent or explicitly unusable.');
  }
  if (!Array.isArray(input.rasterToSourceTransform)
    || input.rasterToSourceTransform.length !== 6
    || !input.rasterToSourceTransform.every(finiteNumber)) {
    throw new Error('OCR requires an exact six-value raster-to-source transform.');
  }
  if (input.ocrRotationRadians !== undefined && !finiteNumber(input.ocrRotationRadians)) {
    throw new Error('OCR normalization rotation must be a finite number of radians.');
  }
  if (!input.crop || !['x0', 'y0', 'x1', 'y1'].every((key) => finiteNumber(input.crop[key]))) {
    throw new Error('OCR requires an exact source-space crop.');
  }
  return bytes;
}

function checkedModelPath() {
  const bytes = fs.readFileSync(OCR_MODEL_PATH);
  const digest = createHash('sha256').update(bytes).digest('hex');
  if (digest !== OCR_MODEL_SHA256) {
    throw new Error('The packaged English OCR model differs from the reviewed SHA-256.');
  }
  return path.dirname(OCR_MODEL_PATH);
}

function transformPoint(transform, x, y) {
  return {
    x: transform[0] * x + transform[2] * y + transform[4],
    y: transform[1] * x + transform[3] * y + transform[5],
  };
}

function transformBox(transform, box) {
  const corners = [
    transformPoint(transform, box.x0, box.y0),
    transformPoint(transform, box.x1, box.y0),
    transformPoint(transform, box.x0, box.y1),
    transformPoint(transform, box.x1, box.y1),
  ];
  return {
    x0: Math.min(...corners.map(({ x }) => x)),
    y0: Math.min(...corners.map(({ y }) => y)),
    x1: Math.max(...corners.map(({ x }) => x)),
    y1: Math.max(...corners.map(({ y }) => y)),
  };
}

function normalizedBoxToInput(box, rotationRadians, inputWidth, inputHeight) {
  const quarterTurns = Math.round(rotationRadians / (Math.PI / 2));
  if (Math.abs(rotationRadians - quarterTurns * Math.PI / 2) > 1e-9) {
    throw new Error('OCR provenance supports exact quarter-turn normalization only.');
  }
  const turns = ((quarterTurns % 4) + 4) % 4;
  const points = [
    { x: box.x0, y: box.y0 }, { x: box.x1, y: box.y0 },
    { x: box.x0, y: box.y1 }, { x: box.x1, y: box.y1 },
  ].map(({ x, y }) => {
    if (turns === 1) return { x: y, y: inputHeight - x };
    if (turns === 2) return { x: inputWidth - x, y: inputHeight - y };
    if (turns === 3) return { x: inputWidth - y, y: x };
    return { x, y };
  });
  return {
    x0: Math.min(...points.map(({ x }) => x)),
    y0: Math.min(...points.map(({ y }) => y)),
    x1: Math.max(...points.map(({ x }) => x)),
    y1: Math.max(...points.map(({ y }) => y)),
  };
}

function wordsFromBlocks(blocks) {
  return (blocks || []).flatMap((block) => block.paragraphs || [])
    .flatMap((paragraph) => paragraph.lines || [])
    .flatMap((line) => line.words || []);
}

async function recognizeOcr(input, options = {}) {
  const policy = { ...DEFAULT_OCR_POLICY, ...(options.policy || {}) };
  const started = Date.now();
  let imageBytes;
  try {
    imageBytes = validateInput(input, policy);
    checkedModelPath();
  } catch (error) {
    return terminalResult(input, 'unavailable', [diagnostic('ocr.unavailable', error.message)], 0);
  }

  if (options.signal?.aborted) {
    return terminalResult(input, 'cancelled', [diagnostic('ocr.cancelled', 'OCR was cancelled before it started.')], 0);
  }

  let worker;
  let terminal = null;
  const terminate = async (status, code, message) => {
    if (terminal) return;
    terminal = { status, code, message };
    if (worker) await worker.terminate().catch(() => {});
  };
  const timeout = setTimeout(
    () => { void terminate('timed_out', 'ocr.timed-out', `OCR exceeded ${policy.maxMillis} ms.`); },
    policy.maxMillis,
  );
  const cancel = () => { void terminate('cancelled', 'ocr.cancelled', 'OCR was cancelled by the user.'); };
  options.signal?.addEventListener('abort', cancel, { once: true });

  try {
    worker = await (options.createWorker || createWorker)(OCR_LANGUAGE, OEM.LSTM_ONLY, {
      langPath: checkedModelPath(),
      gzip: false,
      cacheMethod: 'none',
      logger: options.logger || (() => {}),
    });
    if (terminal) {
      await worker.terminate().catch(() => {});
      return terminalResult(input, terminal.status, [diagnostic(terminal.code, terminal.message)], Date.now() - started);
    }
    await worker.setParameters({ tessedit_pageseg_mode: PSM.SPARSE_TEXT });
    const { data } = await worker.recognize(
      imageBytes,
      { rotateRadians: input.ocrRotationRadians || 0 },
      { text: true, blocks: true },
    );
    if (terminal) {
      return terminalResult(input, terminal.status, [diagnostic(terminal.code, terminal.message)], Date.now() - started);
    }
    const words = wordsFromBlocks(data.blocks).filter((word) => word.text?.trim());
    const characterCount = words.reduce((sum, word) => sum + word.text.length, 0);
    if (words.length > policy.maxWords || characterCount > policy.maxCharacters) {
      await terminate('failed', 'ocr.output-limit', 'OCR output exceeded the reviewed word or character limit.');
      return terminalResult(input, terminal.status, [diagnostic(terminal.code, terminal.message)], Date.now() - started);
    }
    const candidates = words.map((word, index) => ({
      candidateId: `ocr:${input.pageNumber}:${index + 1}`,
      text: word.text,
      sourceBox: transformBox(input.rasterToSourceTransform, normalizedBoxToInput(word.bbox, input.ocrRotationRadians || 0, input.rasterWidth, input.rasterHeight)),
      rasterBox: normalizedBoxToInput(word.bbox, input.ocrRotationRadians || 0, input.rasterWidth, input.rasterHeight),
      confidence: Math.max(0, Math.min(1, Number(word.confidence) / 100)),
      extractionMethod: 'ocr',
      engine: OCR_ENGINE,
      engineVersion: OCR_ENGINE_VERSION,
      modelSha256: OCR_MODEL_SHA256,
      confirmation: 'unconfirmed',
      requiresConfirmation: true,
    }));
    return terminalResult(input, 'completed', [], Date.now() - started, candidates);
  } catch (error) {
    if (terminal) {
      return terminalResult(input, terminal.status, [diagnostic(terminal.code, terminal.message)], Date.now() - started);
    }
    return terminalResult(input, 'failed', [diagnostic('ocr.failed', String(error?.message || error))], Date.now() - started);
  } finally {
    clearTimeout(timeout);
    options.signal?.removeEventListener('abort', cancel);
    if (worker && !terminal) await worker.terminate().catch(() => {});
  }
}

module.exports = {
  DEFAULT_OCR_POLICY,
  OCR_ENGINE_VERSION,
  OCR_MODEL_COMMIT,
  OCR_MODEL_SHA256,
  OCR_SCHEMA,
  recognizeOcr,
};
