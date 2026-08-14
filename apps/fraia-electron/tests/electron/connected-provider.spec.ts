import { test, expect, _electron as electron } from '@playwright/test';
import { spawn, type ChildProcess } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';

import qualification from '../fixtures/connected-qualification-request.json';

const appRoot = path.resolve(__dirname, '..', '..');
const repoRoot = path.resolve(appRoot, '..', '..');
const enabled = process.env.FRAIA_CONNECTED_UI_QUALIFICATION === '1';
const originalDevUserData = path.join(os.homedir(), 'Library', 'Application Support', 'Fraia Dev');

function sourceProvenance() {
  return Object.fromEntries(['main.js', 'preload.js'].map((file) => [file, fs.statSync(path.join(appRoot, file)).mtimeMs]));
}

async function waitForServer(url: string, process: ChildProcess) {
  await expect.poll(async () => {
    if (process.exitCode !== null) throw new Error(`Vite exited with code ${process.exitCode}.`);
    try {
      const response = await fetch(url);
      return response.status < 500;
    } catch {
      return false;
    }
  }, { timeout: 20_000 }).toBe(true);
}

test('current Fraia Dev qualifies the connected provider through the public blank-design journey', async () => {
  test.skip(!enabled, 'Set FRAIA_CONNECTED_UI_QUALIFICATION=1 after the connected prompt audit is green.');
  test.skip(process.platform !== 'darwin', 'The reviewed encrypted Fraia Dev authorization belongs to the current macOS user data.');

  const credentialPath = path.join(originalDevUserData, 'ai', 'credentials.bin');
  expect(fs.existsSync(credentialPath), 'original Fraia Dev encrypted ChatGPT authorization must exist').toBe(true);
  const appdPath = path.resolve(process.env.FRAIA_APPD_PATH ?? '');
  expect(fs.existsSync(appdPath), 'the Electron runner must inject the freshly built fraia-appd').toBe(true);
  const newestAppdSource = Math.max(
    fs.statSync(path.join(repoRoot, 'apps', 'fraia-appd', 'src', 'main.rs')).mtimeMs,
    fs.statSync(path.join(repoRoot, 'apps', 'fraia-appd', 'src', 'conversation_transport.rs')).mtimeMs,
  );
  expect(fs.statSync(appdPath).mtimeMs, 'fraia-appd must not predate its live source').toBeGreaterThanOrEqual(newestAppdSource);

  const port = 56000 + Math.floor(Math.random() * 1000);
  const serverUrl = `http://127.0.0.1:${port}`;
  const vite = spawn(path.join(appRoot, 'node_modules', '.bin', 'vite'), ['--host', '127.0.0.1', '--port', String(port), '--strictPort', '--force'], {
    cwd: appRoot,
    env: process.env,
    stdio: 'pipe',
  });
  let electronApp: Awaited<ReturnType<typeof electron.launch>> | null = null;
  try {
    await waitForServer(serverUrl, vite);
    const environment = { ...process.env };
    delete environment.FRAIA_FAKE_AI_RUNTIME;
    delete environment.FRAIA_FAKE_AI_TURN_DELAY_MS;
    Object.assign(environment, {
      FRAIA_DEV_RUNTIME: '1',
      FRAIA_DEV_APP_DIR: appRoot,
      FRAIA_DEV_SOURCE_PROVENANCE: JSON.stringify(sourceProvenance()),
      FRAIA_USER_DATA_DIR: originalDevUserData,
      VITE_DEV_SERVER_URL: serverUrl,
    });
    electronApp = await electron.launch({ args: ['.'], cwd: appRoot, env: environment });
    const page = await electronApp.firstWindow();
    await page.waitForLoadState('domcontentloaded');
    expect(page.url()).toBe(serverUrl + '/');
    expect(await electronApp.evaluate(({ app }) => app.getPath('userData'))).toBe(originalDevUserData);
    await expect.poll(() => page.evaluate(() => window.fraia.applicationMetadata())).toMatchObject({
      productName: 'Fraia Dev',
      userDataDirectoryName: 'Fraia Dev',
    });
    await expect.poll(() => page.evaluate(() => window.fraia.health())).toMatchObject({ status: 'ok' });

    const providerPreflight = await page.evaluate(async () => {
      try {
        return { catalogue: await window.fraia.aiProviders(), error: null };
      } catch (cause: any) {
        return { catalogue: null, error: cause?.message ?? String(cause) };
      }
    });
    expect(providerPreflight.error, `Fraia AI catalogue preflight failed: ${providerPreflight.error}`).toBeNull();
    expect(providerPreflight.catalogue?.providers).toContainEqual(expect.objectContaining({ id: 'openai-codex', authState: 'connected' }));
    expect(providerPreflight.catalogue?.models).toContainEqual(expect.objectContaining({ providerId: 'openai-codex', modelId: 'gpt-5.6-luna', available: true }));

    await page.getByRole('button', { name: 'New blank model' }).first().click();
    await expect(page.getByTestId('blank-conversation')).toBeVisible();
    await expect(page.getByTestId('conversation-proposal')).toHaveCount(0);
    await expect(page.getByText('Proposed structure')).toHaveCount(0);
    await expect(page.getByRole('button', { name: 'Files' })).toBeVisible();

    await page.getByRole('button', { name: 'Project and design actions' }).click();
    await page.getByRole('menuitem', { name: 'Fraia connection…' }).click();
    const connection = page.getByRole('dialog', { name: 'Fraia AI' });
    await expect(connection.getByText('Ready')).toBeVisible({ timeout: 20_000 });
    await expect(connection.getByText('Signed in securely. Your authorization stays out of project files.')).toBeVisible();
    await connection.getByRole('button', { name: 'Close' }).click();

    const composer = page.getByRole('textbox', { name: 'Conversation message' });
    await composer.pressSequentially(qualification.request, { delay: 12 });
    await page.getByRole('button', { name: 'Send message' }).click();
    await expect(page.getByText(qualification.request)).toBeVisible();
    await expect(page.getByText('Fraia is working…')).toBeVisible({ timeout: 20_000 });
    const proposal = page.getByTestId('conversation-proposal');
    await expect(proposal).toBeVisible({ timeout: 120_000 });
    await expect(page.getByLabel('Fraia AI').last()).not.toBeEmpty();
    await expect(page.getByText('Proposed structure')).toBeVisible();
    await expect(page.getByText(/FRAIA_FAKE/)).toHaveCount(0);
    await expect(page.getByRole('button', { name: 'Accept this direction' })).toBeVisible();
    await page.getByRole('button', { name: 'Accept this direction' }).click();
    await expect(page.getByTestId('proposal-record')).toContainText('Accepted');
    await expect(page.getByText('Current structure')).toBeVisible();

    const screenshotDir = path.join(appRoot, 'tmp', 'connected-qualification');
    fs.mkdirSync(screenshotDir, { recursive: true });
    await page.screenshot({ path: path.join(screenshotDir, 'accepted-current-fraia-dev.png') });
  } finally {
    await electronApp?.close();
    vite.kill('SIGTERM');
  }
});
