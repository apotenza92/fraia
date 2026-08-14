const fs = require('node:fs');
const path = require('node:path');
const { createHash } = require('node:crypto');

const IMPORT_RUNTIME_CONTRACT_VERSION = 'fraia.import-runtime-contract.v1';

const importRuntimeContract = Object.freeze({
  schema: IMPORT_RUNTIME_CONTRACT_VERSION,
  networkPolicy: 'offline-only-no-runtime-downloads',
  targets: [
    'darwin-arm64',
    'darwin-x64',
    'linux-arm64',
    'linux-x64',
    'win32-arm64',
    'win32-x64',
  ],
  importers: Object.freeze({
    pdfIndex: Object.freeze({
      implementation: 'rust-crate',
      parserId: 'lopdf',
      package: 'lopdf',
      version: '0.44.0',
      checksum: '5e2ec995d822e05cabc3f06d196ee43650af3fe4fe38012cacb35e0c3d113b68',
      license: 'MIT',
      licenseFile: 'import-runtime-licenses/LOPDF-MIT.txt',
      licenseSha256: '002e0c25fb21c5270bae08c0c0d206ed07ec60402955a7187f53037fd91e526a',
      packagedAs: 'fraia-appd',
    }),
    pdfRenderer: Object.freeze({
      implementation: 'bundled-browser-worker',
      package: 'pdfjs-dist',
      version: '6.2.108',
      integrity: 'sha512-YxFb+SQcodN2rnX9Tn3dHYlqfb7NjlzzfONPpJd+AKoKtUjEdevTfbC07d5TcczzOK6261auRkP/M8OBHs9vFQ==',
      license: 'Apache-2.0',
      licenseFile: 'import-runtime-licenses/PDFJS-APACHE-2.0.txt',
      licenseSha256: '809fa1ed21450f59827d1e9aec720bbc4b687434fa22283c6cb5dd82a47ab9c0',
      packagedAs: 'vite-renderer-and-dedicated-worker',
    }),
    dxf: Object.freeze({
      implementation: 'in-tree-rust',
      parserId: 'fraia.ascii-dxf.bounded',
      license: 'Fraia repository license',
      packagedAs: 'fraia-appd',
    }),
    ifc: Object.freeze({
      implementation: 'in-tree-rust',
      parserId: 'fraia.ifc-step.bounded',
      license: 'Fraia repository license',
      packagedAs: 'fraia-appd',
    }),
    mesh: Object.freeze({
      implementation: 'in-tree-rust',
      parserId: 'fraia.neutral-mesh.bounded',
      license: 'Fraia repository license',
      packagedAs: 'fraia-appd',
    }),
    ocr: Object.freeze({
      implementation: 'bundled-node-worker-wasm',
      package: 'tesseract.js',
      version: '7.0.0',
      integrity: 'sha512-exPBkd+z+wM1BuMkx/Bjv43OeLBxhL5kKWsz/9JY+DXcXdiBjiAch0V49QR3oAJqCaL5qURE0vx9Eo+G5YE7mA==',
      corePackage: 'tesseract.js-core',
      coreVersion: '7.0.0',
      coreIntegrity: 'sha512-WnNH518NzmbSq9zgTPeoF8c+xmilS8rFIl1YKbk/ptuuc7p6cLNELNuPAzcmsYw450ca6bLa8j3t0VAtq435Vw==',
      license: 'Apache-2.0',
      licenseFile: 'import-runtime-licenses/TESSERACTJS-APACHE-2.0.txt',
      licenseSha256: 'b40930bbcf80744c86c46a12bc9da056641d722716c378f5659b9e555ef833e1',
      modelLanguage: 'eng',
      modelRepository: 'tesseract-ocr/tessdata_fast',
      modelCommit: '65727574dfcd264acbb0c3e07860e4e9e9b22185',
      modelFile: 'ocr-runtime/eng.traineddata',
      modelByteSize: 4113088,
      modelSha256: '7d4322bd2a7749724879683fc3912cb542f19906c83bcc1a52132556427170b2',
      modelLicense: 'Apache-2.0',
      modelLicenseFile: 'import-runtime-licenses/TESSDATA-FAST-APACHE-2.0.txt',
      modelLicenseSha256: 'cfc7749b96f63bd31c3c42b5c471bf756814053e847c10f3eb003417bc523d30',
      coreVariants: Object.freeze([
        'tesseract-core',
        'tesseract-core-lstm',
        'tesseract-core-simd',
        'tesseract-core-simd-lstm',
        'tesseract-core-relaxedsimd',
        'tesseract-core-relaxedsimd-lstm',
      ]),
      coreAssetSha256: Object.freeze({
        'tesseract-core.js': 'a824c1b99a19e122d87e4467fe16aabb56c495d6cc9a08bc58cb8a7342636b43',
        'tesseract-core.wasm': 'c7f5ace62ac0ad065e71e9c6725f1d7cdf82e7eda8fba532cbb9563964da7098',
        'tesseract-core.wasm.js': '0bc6ce3e5fbbd0cd89706cf2fd70960e3372f4f01ee24265b26990808aaeb286',
        'tesseract-core-lstm.js': '6510efc4e8b45c5465df30679b9911ffe0071cd2ee982fa064e6f5136ef2de85',
        'tesseract-core-lstm.wasm': '66b17df6e20c5329a17ffa9c202a47eaa3e32500b253d4c7f38e7f2bc01457c3',
        'tesseract-core-lstm.wasm.js': 'eef5f8b2f8e20e150680b20adaec4a60babafee3adbe8a94583c81fee46e8680',
        'tesseract-core-simd.js': 'da428fd7989ba749855ea16718a83b23e7ce04016fe31866ad2735813efc7133',
        'tesseract-core-simd.wasm': '7d237a13edfeb0fa2f104744fccde0a00e0c076c3e23b7a8fc7af75ec9af2c3e',
        'tesseract-core-simd.wasm.js': '6b61ef4e911b5cf57e656bbfe983d6e2b3711a02dd164154ddda064566e8e09d',
        'tesseract-core-simd-lstm.js': 'e48e2f02ddae3716c8dd24bf41cd290d4efa96892d689cdc4013c2545d63f469',
        'tesseract-core-simd-lstm.wasm': '34e8d50cac216427d86bf397d610fdd9f49492539bbcdfbfccc4eda20c810bea',
        'tesseract-core-simd-lstm.wasm.js': 'c58b46a4c796c0b8afccf77591d5b875b6896b45d402bbce8caa6f5362447b38',
        'tesseract-core-relaxedsimd.js': '716be037611f21b568347421f582f1e1a6456b6d5c3a7c2406c8a2a6c0136427',
        'tesseract-core-relaxedsimd.wasm': '45f8c9b516df326b6ae6b493ed3a6289df5cbd10490e7b6ff8bf5b12ea42d1da',
        'tesseract-core-relaxedsimd.wasm.js': '843074aa5bad1cc6421b74a86201768ced9f244795e4d81435435a61a40ce535',
        'tesseract-core-relaxedsimd-lstm.js': 'a37ac78b707e8d5d3d2e532cc3c4e69b04d127ea44a608f1e7de17640402aa5c',
        'tesseract-core-relaxedsimd-lstm.wasm': '7985c92d4c64e7267d24cadffe1b2a1da6bf8aa55fdcaf953fe94fe122a24545',
        'tesseract-core-relaxedsimd-lstm.wasm.js': '861a536cf9ef8e63cb644d57bab39c388f37f7d6b6f60024b741c5f6b39a59b3',
      }),
      packagedAs: 'electron-main-node-worker-and-local-wasm',
      behavior: 'typed-unconfirmed-spatial-text-candidates-no-runtime-downloads',
    }),
  }),
});

function validateImportRuntimeSources(appRoot) {
  const repositoryRoot = path.resolve(appRoot, '..', '..');
  const packageLock = JSON.parse(fs.readFileSync(path.join(appRoot, 'package-lock.json'), 'utf8'));
  const cargoLock = fs.readFileSync(path.join(repositoryRoot, 'Cargo.lock'), 'utf8');
  const pdfSource = fs.readFileSync(path.join(repositoryRoot, 'crates', 'fraia-core', 'src', 'pdf_ingest.rs'), 'utf8');
  const appdSource = fs.readFileSync(path.join(repositoryRoot, 'apps', 'fraia-appd', 'src', 'main.rs'), 'utf8');
  const parserSources = {
    dxf: fs.readFileSync(path.join(repositoryRoot, 'crates', 'fraia-core', 'src', 'dxf_ingest.rs'), 'utf8'),
    ifc: fs.readFileSync(path.join(repositoryRoot, 'crates', 'fraia-core', 'src', 'ifc_ingest.rs'), 'utf8'),
    mesh: fs.readFileSync(path.join(repositoryRoot, 'crates', 'fraia-core', 'src', 'mesh_ingest.rs'), 'utf8'),
  };
  const pdfjs = packageLock.packages['node_modules/pdfjs-dist'];
  const expectedPdfjs = importRuntimeContract.importers.pdfRenderer;
  if (
    pdfjs?.version !== expectedPdfjs.version
    || pdfjs?.integrity !== expectedPdfjs.integrity
    || pdfjs?.license !== expectedPdfjs.license
  ) {
    throw new Error('PDF.js package bytes or licence differ from the reviewed import contract.');
  }
  const lopdf = importRuntimeContract.importers.pdfIndex;
  const escapedChecksum = lopdf.checksum.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  const lopdfPattern = new RegExp(
    `name = "${lopdf.package}"\\nversion = "${lopdf.version}"\\nsource = "[^"]+"\\nchecksum = "${escapedChecksum}"`,
  );
  if (!lopdfPattern.test(cargoLock) || !pdfSource.includes(`PDF_PARSER_ID: &str = "${lopdf.parserId}"`)) {
    throw new Error('lopdf source identity differs from the reviewed import contract.');
  }
  for (const importer of [lopdf, expectedPdfjs]) {
    const license = fs.readFileSync(path.join(appRoot, importer.licenseFile));
    const digest = createHash('sha256').update(license).digest('hex');
    if (digest !== importer.licenseSha256) {
      throw new Error(`${importer.package} packaged licence bytes differ from the reviewed contract.`);
    }
  }
  const ocr = importRuntimeContract.importers.ocr;
  const tesseract = packageLock.packages[`node_modules/${ocr.package}`];
  const core = packageLock.packages[`node_modules/${ocr.corePackage}`];
  if (tesseract?.version !== ocr.version || tesseract?.integrity !== ocr.integrity
    || core?.version !== ocr.coreVersion || core?.integrity !== ocr.coreIntegrity) {
    throw new Error('Tesseract.js package or core bytes differ from the reviewed OCR contract.');
  }
  for (const [file, expected] of [
    [ocr.licenseFile, ocr.licenseSha256],
    [ocr.modelLicenseFile, ocr.modelLicenseSha256],
    [ocr.modelFile, ocr.modelSha256],
  ]) {
    const bytes = fs.readFileSync(path.join(appRoot, file));
    if (createHash('sha256').update(bytes).digest('hex') !== expected) {
      throw new Error(`${file} differs from the reviewed OCR contract.`);
    }
  }
  if (fs.statSync(path.join(appRoot, ocr.modelFile)).size !== ocr.modelByteSize) {
    throw new Error('The English OCR model byte size differs from the reviewed contract.');
  }
  const coreRoot = path.join(appRoot, 'node_modules', ocr.corePackage);
  for (const [asset, expected] of Object.entries(ocr.coreAssetSha256)) {
    const assetPath = path.join(coreRoot, asset);
    if (!fs.statSync(assetPath, { throwIfNoEntry: false })?.isFile()
      || createHash('sha256').update(fs.readFileSync(assetPath)).digest('hex') !== expected) {
      throw new Error(`The reviewed OCR core payload differs at ${asset}.`);
    }
  }
  for (const [name, source] of Object.entries(parserSources)) {
    if (!source.includes(importRuntimeContract.importers[name].parserId)) {
      throw new Error(`${name} parser identity differs from the reviewed import contract.`);
    }
  }
  const ocrSource = fs.readFileSync(path.join(appRoot, 'ocr-runtime.cjs'), 'utf8');
  if (!ocrSource.includes(ocr.modelCommit) || !ocrSource.includes(ocr.modelSha256)
    || !ocrSource.includes("confirmation: 'unconfirmed'")
    || !ocrSource.includes("requiresConfirmation: true")) {
    throw new Error('OCR provenance and unconfirmed-candidate behavior differ from the reviewed contract.');
  }
  const productionSources = [pdfSource, ocrSource, ...Object.values(parserSources)].join('\n');
  if (/https?:\/\/|\b(?:curl|wget)\b|reqwest::|ureq::/.test(productionSources)) {
    throw new Error('An import parser contains an undeclared production network path.');
  }
  return importRuntimeContract;
}

function validateBuiltImportAssets(appRoot) {
  const dist = path.join(appRoot, 'dist');
  const assets = fs.readdirSync(path.join(dist, 'assets'));
  const workers = assets.filter((name) => /^pdf\.worker\.min-[A-Za-z0-9_-]+\.mjs$/.test(name));
  if (workers.length !== 1) {
    throw new Error(`Expected one bundled PDF.js worker, found ${workers.length}.`);
  }
  const worker = fs.readFileSync(path.join(dist, 'assets', workers[0]), 'utf8');
  if (/(?:importScripts|fetch)\s*\(\s*['"]https?:\/\//.test(worker)) {
    throw new Error('The bundled PDF.js worker contains a production network loader.');
  }
  const ocr = importRuntimeContract.importers.ocr;
  const model = path.join(appRoot, ocr.modelFile);
  if (!fs.statSync(model, { throwIfNoEntry: false })?.isFile()
    || createHash('sha256').update(fs.readFileSync(model)).digest('hex') !== ocr.modelSha256) {
    throw new Error('The built OCR model is missing or differs from the reviewed contract.');
  }
  return {
    worker: workers[0],
    byteSize: Buffer.byteLength(worker),
    ocrModelSha256: ocr.modelSha256,
    ocrCoreVariants: ocr.coreVariants.length,
  };
}

module.exports = {
  IMPORT_RUNTIME_CONTRACT_VERSION,
  importRuntimeContract,
  validateBuiltImportAssets,
  validateImportRuntimeSources,
};
