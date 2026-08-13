import { expect, test, _electron as electron } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';

const allowedConsoleWarnings = [
  /THREE\.Clock: This module has been deprecated/,
  /Electron Security Warning \(Insecure Content-Security-Policy\)/,
];
const deterministicLinuxRenderingArgs = process.platform === 'linux'
  ? ['--use-gl=angle', '--use-angle=swiftshader', '--enable-unsafe-swiftshader']
  : [];

test('desktop shell opens a sparse conversation workspace with read-only preview and one manual revision handoff', async () => {
  const appRoot = process.cwd();
  const temporaryRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'fraia-conversation-e2e-'));
  const projectDir = path.join(temporaryRoot, 'project');
  const movedProjectDir = path.join(temporaryRoot, 'moved-project');
  const userDataDir = path.join(temporaryRoot, 'user-data');
  fs.mkdirSync(projectDir, { recursive: true });
  fs.mkdirSync(userDataDir, { recursive: true });

  expect(fs.existsSync(path.join(appRoot, 'dist', 'index.html'))).toBe(true);

  const consoleProblems: string[] = [];
  const pageErrors: string[] = [];
  let electronApp = await electron.launch({
    args: [...deterministicLinuxRenderingArgs, '.', `--user-data-dir=${userDataDir}`],
    cwd: appRoot,
    env: {
      ...process.env,
      FRAIA_DEFAULT_PROJECT_DIR: projectDir,
      FRAIA_USER_DATA_DIR: userDataDir,
      FRAIA_DISABLE_MANAGED_CCX_BOOTSTRAP: '1',
    },
  });

  try {
    let page = await electronApp.firstWindow();
    page.on('console', (message) => {
      if (message.type() === 'error' || message.type() === 'warning') {
        const text = message.text();
        if (!allowedConsoleWarnings.some((pattern) => pattern.test(text))) consoleProblems.push(`${message.type()}: ${text}`);
      }
    });
    page.on('pageerror', (error) => pageErrors.push(error.message));

    await page.waitForLoadState('domcontentloaded');
    await expect(page.locator('[data-slot=menubar]')).toBeVisible();
    await expect(page.getByTestId('conversation-workspace-shell')).toBeVisible();
    await expect(page.getByTestId('empty-workspace')).toBeVisible();
    await expect.poll(async () => {
      try {
        return await page.evaluate(() => window.fraia.health());
      } catch {
        return null;
      }
    }, { timeout: 70_000 }).toMatchObject({ status: 'ok' });
    await electronApp.evaluate(({ dialog }, selectedDirectory) => {
      dialog.showOpenDialog = async () => ({ canceled: false, filePaths: [selectedDirectory] });
    }, projectDir);
    await page.getByRole('button', { name: 'New blank model' }).first().click();
    await expect(page.getByTestId('conversation-workspace')).toBeVisible();
    await expect(page.getByTestId('conversation-workspace').getByText('Overall framing', { exact: true })).toBeVisible();
    await expect(page.getByText('Structural preview', { exact: true })).toBeVisible();
    await expect(page.getByRole('button', { name: 'Open in editor' })).toBeVisible();
    await expect.poll(() => fs.existsSync(path.join(projectDir, '.fraia', 'workspace.sqlite'))).toBe(true);
    expect(fs.existsSync(path.join(userDataDir, 'conversations.sqlite'))).toBe(false);
    const brief = page.getByTestId('project-brief');
    await brief.getByRole('button', { name: 'Add brief' }).click();
    await expect(brief.getByRole('button', { name: 'Hide brief' })).toBeVisible();
    await brief.locator('#brief-building-type').fill('workshop');
    await brief.getByRole('button', { name: 'Save brief' }).click();

    await expect(page.getByRole('navigation', { name: 'Design workflow' })).toHaveCount(0);
    await expect(page.getByText('Base Model', { exact: true })).toHaveCount(0);
    await expect(page.getByText('Design Options', { exact: true })).toHaveCount(0);
    await expect(page.getByText('Analysis & Comparison', { exact: true })).toHaveCount(0);
    await expect(page.locator('canvas[data-fraia-canvas-role="viewport-webgl"]')).toHaveCount(1);
    await expect(page.locator('canvas[data-fraia-canvas-role="selection-overlay"]')).toHaveCount(1);

    await page.getByRole('button', { name: 'Accept this direction' }).click();
    await expect(page.getByText('This direction is now the current design. We can analyse it or refine it.')).toBeVisible();
    await page.getByTestId('run-analysis').click();
    await expect(page.getByTestId('analysis-result-card').getByText('Analysis complete', { exact: true })).toBeVisible();
    await page.getByRole('button', { name: 'Explore another' }).click();
    await page.getByRole('button', { name: 'Analyse candidate' }).click();
    await expect(page.getByTestId('comparison-evidence-boundary')).toHaveText('Both directions are ready to compare.');
    await page.getByRole('button', { name: 'Compare evidence' }).click();
    await expect(page.getByTestId('comparison-metrics')).toBeVisible();
    await page.getByRole('button', { name: 'Inspect' }).click();
    await expect(page.getByRole('dialog', { name: 'Inspect structural preview' })).toBeVisible();
    await expect(page.getByText(/Inspection does not edit the model/)).toBeVisible();
    await page.getByRole('dialog', { name: 'Inspect structural preview' }).getByRole('button', { name: 'Open in editor' }).click();
    await expect(page.getByTestId('working-copy-panel')).toBeVisible();
    const nodeX = page.getByRole('spinbutton', { name: 'Node x coordinate in metres' });
    const initialNodePosition = await page.getByTestId('working-copy-node-position').textContent();
    await nodeX.fill('1.5');
    await page.getByRole('button', { name: 'Move selected node' }).click();
    await expect(page.getByTestId('working-copy-node-position')).not.toHaveText(initialNodePosition ?? '');
    await page.getByRole('button', { name: 'Record manual change' }).click();
    await expect(page.getByText('2 pending edits')).toBeVisible();
    await page.getByRole('button', { name: 'Return to conversation' }).click();
    await expect(page.getByTestId('stale-evidence')).toHaveText('Stale evidence');
    await expect(page.getByText(/Your manual changes are back in the conversation/)).toBeVisible();
    await page.getByRole('button', { name: 'Run analysis' }).click();
    await expect(page.getByTestId('analysis-result-card').last()).toBeVisible();
    await expect(page.getByText('Analysis complete', { exact: true }).last()).toBeVisible();
    await page.getByTestId('view-analysis').click();
    await expect.poll(() => page.evaluate(() => {
      const cards = document.querySelectorAll('[data-testid="analysis-result-card"]');
      const latest = cards.item(cards.length - 1);
      if (!latest) return false;
      const bounds = latest.getBoundingClientRect();
      return bounds.top >= 0 && bounds.bottom <= window.innerHeight;
    })).toBe(true);
    await expect(page.getByTestId('stale-evidence')).toHaveCount(0);

    const accessibility = await new AxeBuilder({ page }).setLegacyMode().analyze();
    expect(pageErrors, 'unexpected renderer exceptions').toEqual([]);
    expect(consoleProblems, 'unexpected renderer warnings or errors').toEqual([]);
    expect(accessibility.violations, 'axe accessibility violations').toEqual([]);

    await electronApp.close();
    fs.renameSync(projectDir, movedProjectDir);
    electronApp = await electron.launch({
      args: [...deterministicLinuxRenderingArgs, '.', `--user-data-dir=${userDataDir}`],
      cwd: appRoot,
      env: {
        ...process.env,
        FRAIA_DEFAULT_PROJECT_DIR: movedProjectDir,
        FRAIA_USER_DATA_DIR: userDataDir,
        FRAIA_DISABLE_MANAGED_CCX_BOOTSTRAP: '1',
      },
    });
    page = await electronApp.firstWindow();
    await page.waitForLoadState('domcontentloaded');
    await expect(page.getByTestId('empty-workspace')).toBeVisible();
    await electronApp.evaluate(({ dialog }, selectedProjectFile) => {
      dialog.showOpenDialog = async () => ({ canceled: false, filePaths: [selectedProjectFile] });
    }, path.join(movedProjectDir, 'fraia.project.json'));
    await page.getByRole('button', { name: 'Open model' }).click();
    await page.getByRole('button', { name: 'Add brief' }).click();
    await expect(page.locator('#brief-building-type')).toHaveValue('workshop');
    await expect(page.getByRole('button', { name: 'Accept this direction' })).toHaveCount(0);
    await expect(page.getByText('Your current design was restored.')).toBeVisible();
  } finally {
    await electronApp.close();
    fs.rmSync(temporaryRoot, { recursive: true, force: true });
  }
});
