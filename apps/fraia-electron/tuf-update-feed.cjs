const fs = require('node:fs');
const http = require('node:http');
const path = require('node:path');
const { Updater } = require('tuf-js');

function isLoopbackHost(hostname) {
  return ['127.0.0.1', '::1', 'localhost'].includes(hostname);
}

function validateRepositoryUrl(value, { allowLoopbackHttp = false } = {}) {
  const parsed = new URL(value);
  if (parsed.protocol !== 'https:' && !(allowLoopbackHttp && parsed.protocol === 'http:' && isLoopbackHost(parsed.hostname))) {
    throw new Error('Fraia TUF repositories must use HTTPS (loopback HTTP is test-only).');
  }
  return parsed.toString().replace(/\/$/, '');
}

function validateTargetName(value) {
  if (
    typeof value !== 'string'
    || !value
    || value !== path.posix.basename(value)
    || value.includes('\\')
    || value.includes('\0')
  ) {
    throw new Error(`Unsafe TUF update target name: ${value}`);
  }
  return value;
}

function initializeTrustedRoot({ embeddedRootPath, metadataDir }) {
  const trustedRootPath = path.join(metadataDir, 'root.json');
  fs.mkdirSync(metadataDir, { recursive: true });
  if (fs.existsSync(trustedRootPath)) {
    if (!fs.statSync(trustedRootPath).isFile()) {
      throw new Error('The persisted Fraia TUF root is not a regular file.');
    }
    return { initialized: false, trustedRootPath };
  }
  if (!embeddedRootPath || !fs.statSync(embeddedRootPath, { throwIfNoEntry: false })?.isFile()) {
    throw new Error('Fraia has no embedded TUF trust root.');
  }
  fs.copyFileSync(embeddedRootPath, trustedRootPath, fs.constants.COPYFILE_EXCL);
  return { initialized: true, trustedRootPath };
}

function listen(server) {
  return new Promise((resolve, reject) => {
    const onError = (error) => {
      server.off('listening', onListening);
      reject(error);
    };
    const onListening = () => {
      server.off('error', onError);
      resolve();
    };
    server.once('error', onError);
    server.once('listening', onListening);
    server.listen(0, '127.0.0.1');
  });
}

function close(server) {
  return new Promise((resolve, reject) => {
    server.close((error) => {
      if (error) reject(error);
      else resolve();
    });
  });
}

async function createTufVerifiedUpdateFeed({
  embeddedRootPath,
  repositoryUrl,
  targetName,
  trustDir,
  allowLoopbackHttp = false,
  UpdaterClass = Updater,
} = {}) {
  const normalizedRepositoryUrl = validateRepositoryUrl(repositoryUrl, { allowLoopbackHttp });
  const normalizedTargetName = validateTargetName(targetName);
  const metadataDir = path.join(trustDir, 'metadata');
  const targetDir = path.join(trustDir, 'targets');
  fs.mkdirSync(targetDir, { recursive: true });
  const trust = initializeTrustedRoot({ embeddedRootPath, metadataDir });

  const updater = new UpdaterClass({
    metadataBaseUrl: `${normalizedRepositoryUrl}/metadata`,
    targetBaseUrl: `${normalizedRepositoryUrl}/targets`,
    metadataDir,
    targetDir,
    config: { userAgent: 'Fraia desktop updater' },
  });
  await updater.refresh();
  const targetInfo = await updater.getTargetInfo(normalizedTargetName);
  if (!targetInfo) {
    throw new Error(`The signed Fraia update repository has no ${normalizedTargetName} target.`);
  }

  const targetPath = path.join(targetDir, normalizedTargetName);
  const temporaryTargetPath = `${targetPath}.${process.pid}.tmp`;
  try {
    await updater.downloadTarget(targetInfo, temporaryTargetPath);
    fs.renameSync(temporaryTargetPath, targetPath);
  } finally {
    fs.rmSync(temporaryTargetPath, { force: true });
  }
  const targetBytes = fs.readFileSync(targetPath);

  const requestPath = `/${encodeURIComponent(normalizedTargetName)}`;
  const server = http.createServer((request, response) => {
    let pathname;
    try {
      pathname = new URL(request.url, 'http://127.0.0.1').pathname;
    } catch {
      response.writeHead(400).end();
      return;
    }
    if (!['GET', 'HEAD'].includes(request.method) || pathname !== requestPath) {
      response.writeHead(404, { 'Cache-Control': 'no-store' }).end();
      return;
    }
    response.writeHead(200, {
      'Cache-Control': 'no-store',
      'Content-Length': targetBytes.length,
      'Content-Type': 'application/yaml',
    });
    response.end(request.method === 'HEAD' ? undefined : targetBytes);
  });
  await listen(server);
  const address = server.address();
  if (!address || typeof address === 'string') {
    await close(server);
    throw new Error('Fraia could not start its verified local update feed.');
  }

  return {
    close: () => close(server),
    feedUrl: `http://127.0.0.1:${address.port}`,
    targetPath,
    trustInitialized: trust.initialized,
    trustedRootPath: trust.trustedRootPath,
  };
}

module.exports = {
  createTufVerifiedUpdateFeed,
  initializeTrustedRoot,
  validateRepositoryUrl,
  validateTargetName,
};
