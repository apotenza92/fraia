import { expect, test, _electron as electron } from '@playwright/test';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';

const renderingArgs = process.platform === 'linux'
  ? ['--use-gl=angle', '--use-angle=swiftshader', '--enable-unsafe-swiftshader']
  : [];

test('accepted typed geometry reopens as an editable authoritative scene', async () => {
  test.setTimeout(30_000);
  const appRoot = process.cwd();
  const temporaryRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'fraia-accepted-reopen-'));
  const projectDir = path.join(temporaryRoot, 'project');
  const userDataDir = path.join(temporaryRoot, 'user-data');
  const environment = {
    ...process.env,
    FRAIA_DEFAULT_PROJECT_DIR: projectDir,
    FRAIA_USER_DATA_DIR: userDataDir,
    FRAIA_DISABLE_MANAGED_CCX_BOOTSTRAP: '1',
    FRAIA_FAKE_AI_RUNTIME: '1',
  };
  let app = await electron.launch({ args: [...renderingArgs, '.', `--user-data-dir=${userDataDir}`], cwd: appRoot, env: environment });
  try {
    let page = await app.firstWindow();
    await page.evaluate(() => window.fraia.aiStartOAuth({ providerId: 'openai-codex' }));
    await page.getByRole('button', { name: 'New blank model' }).first().click();
    const managedRoot = path.join(userDataDir, 'unsaved-projects');
    await expect.poll(() => fs.existsSync(managedRoot) ? fs.readdirSync(managedRoot)
      .map((entry) => path.join(managedRoot, entry))
      .find((entry) => fs.existsSync(path.join(entry, 'fraia.project.json'))) : undefined).toEqual(expect.any(String));
    const managedProject = fs.readdirSync(managedRoot)
      .map((entry) => path.join(managedRoot, entry))
      .find((entry) => fs.existsSync(path.join(entry, 'fraia.project.json')))!;
    const request = 'Use the confirmed six metre span and simple supports from this test request.';
    await page.getByRole('textbox', { name: 'Conversation message' }).fill(request);
    await page.getByRole('button', { name: 'Send message' }).click();
    await expect(page.getByTestId('conversation-proposal')).toBeVisible();
    await page.getByRole('button', { name: 'Accept this direction' }).click();
    await expect(page.getByTestId('proposal-record')).toContainText('Accepted');
    await app.close();

    app = await electron.launch({ args: [...renderingArgs, '.', `--user-data-dir=${userDataDir}`], cwd: appRoot, env: environment });
    page = await app.firstWindow();
    await app.evaluate(({ dialog }, manifest) => { dialog.showOpenDialog = async () => ({ canceled: false, filePaths: [manifest] }); }, path.join(managedProject, 'fraia.project.json'));
    await page.getByRole('button', { name: 'Open model' }).click();
    await expect(page.getByText('Your current design was restored.')).toBeVisible();
    await expect(page.getByTestId('artefact-preview')).toBeVisible();
    await page.getByRole('button', { name: 'Open in editor' }).click();
    await expect(page.getByText('Precision editor')).toBeVisible();
    await page.getByRole('button', { name: 'Record manual change' }).click();
    await expect(page.getByText('1 pending edit')).toBeVisible();
  } finally {
    await app.close().catch(() => {});
  }
});
