import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { resolve } from 'node:path';
import test from 'node:test';

const root = resolve(import.meta.dirname, '..');
const requireFromApp = createRequire(resolve(root, 'apps/fraia-electron/package.json'));
const { JSDOM } = requireFromApp('jsdom');
const html = readFileSync(resolve(root, 'docs/index.html'), 'utf8');
const readme = readFileSync(resolve(root, 'README.md'), 'utf8');
const stableIcon = readFileSync(resolve(root, 'assets/fraia-icon.svg'), 'utf8');
const publishedStableIcon = readFileSync(resolve(root, 'docs/assets/icons/icon.svg'), 'utf8');
const betaIcon = readFileSync(resolve(root, 'assets/fraia-icon-beta.svg'), 'utf8');
const publishedBetaIcon = readFileSync(resolve(root, 'docs/assets/icons/beta/icon.svg'), 'utf8');
const productDescription = 'Fraia is a desktop workbench for shaping structural schemes, understanding how they behave, and developing better-informed design options.';

async function loadDownloadPage({ architecture = '', platform = '', releases = [], userAgent = '' } = {}) {
  const dom = new JSDOM(html, {
    beforeParse(window) {
      Object.defineProperty(window.navigator, 'platform', { configurable: true, value: platform });
      Object.defineProperty(window.navigator, 'userAgent', { configurable: true, value: userAgent });
      Object.defineProperty(window.navigator, 'userAgentData', {
        configurable: true,
        value: platform || architecture ? { architecture, platform } : undefined
      });
      window.fetch = async () => ({ json: async () => releases, ok: true });
      window.matchMedia = () => ({ addEventListener() {}, matches: false, removeEventListener() {} });
    },
    runScripts: 'dangerously',
    url: 'https://apotenza92.github.io/fraia/'
  });

  await new Promise((resolveTick) => dom.window.setTimeout(resolveTick, 5));
  return dom;
}

function hero(dom) {
  const link = dom.window.document.getElementById('hero-download-button');
  return {
    disabled: link.getAttribute('aria-disabled'),
    href: link.href,
    label: dom.window.document.getElementById('hero-download-label').textContent
  };
}

test('keeps the public product description in sync with the README', async () => {
  const dom = await loadDownloadPage();
  assert.equal(dom.window.document.querySelector('.subtitle').textContent, productDescription);
  assert.match(readme, new RegExp(productDescription.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')));
});

test('publishes the exact maintained stable and beta icon sources', () => {
  assert.equal(publishedStableIcon, stableIcon);
  assert.equal(publishedBetaIcon, betaIcon);
});

test('recommends the signed macOS disk image for Apple Silicon', async () => {
  const dom = await loadDownloadPage({ architecture: 'arm64', platform: 'macOS' });
  assert.match(hero(dom).label, /Download Fraia for Apple Silicon Mac/);
  assert.match(hero(dom).href, /Fraia-macOS-arm64\.dmg$/);
  assert.equal(dom.window.document.getElementById('unsigned-package-notice').hidden, true);
});

test('recommends the Intel macOS package when x64 is detected', async () => {
  const dom = await loadDownloadPage({ architecture: 'x86', platform: 'macOS' });
  assert.match(hero(dom).label, /Download Fraia for Intel Mac/);
  assert.match(hero(dom).href, /Fraia-macOS-x64\.dmg$/);
});

test('recommends Windows ARM64 and shows the unsigned installer disclosure', async () => {
  const dom = await loadDownloadPage({ architecture: 'arm64', platform: 'Windows' });
  assert.match(hero(dom).label, /Download Fraia for Windows ARM64/);
  assert.match(hero(dom).href, /Fraia-Windows-arm64-Setup\.exe$/);
  assert.equal(dom.window.document.getElementById('unsigned-package-notice').hidden, false);
});

test('supports Linux format and architecture selection', async () => {
  const dom = await loadDownloadPage({ architecture: 'x86', platform: 'Linux' });
  const document = dom.window.document;
  assert.match(hero(dom).href, /Fraia-Linux-x64\.AppImage$/);
  document.getElementById('format-deb').click();
  document.getElementById('arch-arm64').click();
  assert.match(hero(dom).label, /Fraia \.deb for Ubuntu \/ Debian ARM64/);
  assert.match(hero(dom).href, /Fraia-Linux-arm64\.deb$/);
});

test('asks an unknown system to choose a download', async () => {
  const dom = await loadDownloadPage();
  assert.equal(hero(dom).label, 'Choose your download');
  assert.equal(hero(dom).disabled, 'true');
});

test('uses a stable release that promotes a newer Fraia Beta identity', async () => {
  const dom = await loadDownloadPage({
    architecture: 'arm64',
    platform: 'macOS',
    releases: [
      {
        assets: [
          { name: 'Fraia-macOS-arm64.dmg', size: 120_000_000 },
          { name: 'Fraia-Beta-macOS-arm64.dmg', size: 121_000_000 }
        ],
        draft: false,
        prerelease: false,
        tag_name: 'v0.0.6'
      },
      {
        assets: [{ name: 'Fraia-Beta-macOS-arm64.dmg', size: 110_000_000 }],
        draft: false,
        prerelease: true,
        tag_name: 'v0.0.4-beta.1'
      }
    ]
  });
  const document = dom.window.document;
  document.getElementById('channel-beta').click();
  assert.equal(document.getElementById('version-display').textContent, 'v0.0.6');
  assert.match(hero(dom).href, /releases\/download\/v0\.0\.6\/Fraia-Beta-macOS-arm64\.dmg$/);
  assert.match(hero(dom).label, /121 MB/);
});

test('uses the maintained blue stable and purple beta colour schemes', async () => {
  const dom = await loadDownloadPage({ architecture: 'arm64', platform: 'macOS' });
  const document = dom.window.document;
  const styles = () => dom.window.getComputedStyle(document.body);

  assert.equal(styles().getPropertyValue('--accent').trim(), '#244bc1');
  assert.match(styles().getPropertyValue('--background'), /#edf2ff/);

  document.getElementById('channel-beta').click();

  assert.equal(styles().getPropertyValue('--accent').trim(), '#6f2aaa');
  assert.match(styles().getPropertyValue('--background'), /#f3e9ff/);
  assert.match(document.getElementById('app-icon').src, /assets\/icons\/beta\/icon\.svg$/);
});

test('selects a higher prerelease for beta without changing stable', async () => {
  const dom = await loadDownloadPage({
    architecture: 'arm64',
    platform: 'macOS',
    releases: [
      {
        assets: [
          { name: 'Fraia-macOS-arm64.dmg', size: 120_000_000 },
          { name: 'Fraia-Beta-macOS-arm64.dmg', size: 121_000_000 }
        ],
        draft: false,
        prerelease: false,
        tag_name: 'v0.0.6'
      },
      {
        assets: [{ name: 'Fraia-Beta-macOS-arm64.dmg', size: 130_000_000 }],
        draft: false,
        prerelease: true,
        tag_name: 'v0.0.7-beta.2'
      }
    ]
  });
  const document = dom.window.document;
  assert.equal(document.getElementById('version-display').textContent, 'v0.0.6');
  document.getElementById('channel-beta').click();
  assert.equal(document.getElementById('version-display').textContent, 'v0.0.7-beta.2');
  assert.match(hero(dom).href, /releases\/download\/v0\.0\.7-beta\.2\/Fraia-Beta-macOS-arm64\.dmg$/);
});

test('links every selected release to checksums and provenance', async () => {
  const dom = await loadDownloadPage({
    releases: [{
      assets: [
        { name: 'Fraia-macOS-arm64.dmg', size: 120_000_000 },
        { name: 'Fraia-Beta-macOS-arm64.dmg', size: 121_000_000 }
      ],
      draft: false,
      prerelease: false,
      tag_name: 'v0.0.6'
    }]
  });
  const document = dom.window.document;
  assert.match(document.getElementById('checksums-link').href, /v0\.0\.6\/SHA256SUMS$/);
  assert.match(document.getElementById('release-link').href, /releases\/tag\/v0\.0\.6$/);
});
