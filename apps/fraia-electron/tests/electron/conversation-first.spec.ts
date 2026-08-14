import { expect, test, _electron as electron } from '@playwright/test';
import type { Locator } from '@playwright/test';
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

test('desktop shell keeps blank conversation truthful and preserves project files with isolated design references', async () => {
  test.setTimeout(process.env.CI ? 300_000 : 180_000);
  const journeyStartedAt = Date.now();
  const checkpoint = (phase: string) => {
    console.log(`[conversation-first] ${phase} at ${Date.now() - journeyStartedAt} ms`);
  };
  const appRoot = process.cwd();
  const temporaryRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'fraia-conversation-e2e-'));
  const projectDir = path.join(temporaryRoot, 'project');
  const movedProjectDir = path.join(temporaryRoot, 'moved-project');
  const userDataDir = path.join(temporaryRoot, 'user-data');
  const sourceFixture = path.join(appRoot, 'tests', 'fixtures', 'architectural-drawings.pdf');
  const scannedPdfFixture = path.join(appRoot, 'tests', 'fixtures', 'scanned-architectural-drawing.pdf');
  const dxfFixture = path.join(appRoot, 'tests', 'fixtures', 'small-frame.dxf');
  const ifcFixture = path.join(appRoot, 'tests', 'fixtures', 'architect-reference.ifc');
  const meshFixture = path.join(appRoot, 'tests', 'fixtures', 'reference-frame.obj');
  const visualRoot = path.join(appRoot, 'tmp', 'visual-matrix');
  const visualSizes = process.env.CI
    ? [['minimum', 900, 600] as const]
    : [['minimum', 900, 600] as const, ['default', 1152, 768] as const, ['large', 1440, 960] as const];
  fs.mkdirSync(visualRoot, { recursive: true });
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
      FRAIA_FAKE_AI_RUNTIME: '1',
      FRAIA_FAKE_AI_TURN_DELAY_MS: '650',
      FRAIA_TEST_ANALYSIS_DELAY_MS: '2000',
      FRAIA_TEST_ANALYSIS_FAILURE: '1',
    },
  });
  electronApp.process().stdout?.on('data', (chunk) => process.stdout.write(`[electron] ${chunk}`));
  electronApp.process().stderr?.on('data', (chunk) => process.stderr.write(`[electron] ${chunk}`));
  checkpoint('first application launched');

  try {
    let page = await electronApp.firstWindow();
    async function scrollStable(locator: Locator) {
      await expect.poll(async () => {
        try {
          await locator.scrollIntoViewIfNeeded({ timeout: 1_000 });
          return true;
        } catch {
          return false;
        }
      }, { timeout: 5_000 }).toBe(true);
    }
    async function captureVisualState(name: string) {
      for (const [sizeName, width, height] of visualSizes) {
        await electronApp.evaluate(({ BrowserWindow }, size) => BrowserWindow.getAllWindows()[0]?.setContentSize(size.width, size.height), { width, height });
        await expect.poll(() => page.evaluate(() => innerWidth)).toBe(width);
        await page.waitForTimeout(350);
        if (name === 'pending-proposal') await scrollStable(page.getByTestId('conversation-proposal'));
        if (name === 'analysis-result') await scrollStable(page.getByTestId('analysis-result-card'));
        if (name.startsWith('analysis-attempt-')) await scrollStable(page.getByTestId('analysis-attempt'));
        if (name === 'stale-analysis') await scrollStable(page.getByTestId('stale-evidence'));
        if (name === 'dxf-content-selection') await scrollStable(page.getByRole('checkbox', { name: 'Select layer FRAME' }));
        if (name === 'dxf-explicit-relation') await scrollStable(page.getByRole('checkbox', { name: 'I checked the drawing view and scale.' }));
        if (name === 'dxf-interpretation-unconfirmed') await scrollStable(page.getByText('Fraia inferred').first());
        if (name === 'ifc-content-selection') await scrollStable(page.getByRole('checkbox', { name: 'Select storey Level 2' }));
        const layout = await page.evaluate(() => {
          const root = document.documentElement;
          const dialogs = [...document.querySelectorAll<HTMLElement>('[role="dialog"]')].filter((element) => element.offsetParent !== null).map((element) => {
            const bounds = element.getBoundingClientRect();
            return { left: bounds.left, top: bounds.top, right: bounds.right, bottom: bounds.bottom };
          });
          const undersizedControls = [...document.querySelectorAll<HTMLElement>('button:not([disabled]), [role="tab"]')].filter((element) => {
            if (element.offsetParent === null) return false;
            const bounds = element.getBoundingClientRect();
            return bounds.width < 24 || bounds.height < 24;
          }).length;
          const tabCloseIntersections = [...document.querySelectorAll<HTMLElement>('[role="tab"]')].filter((tab) => {
            const label = tab.querySelector<HTMLElement>('[data-document-tab-label]');
            const close = [...document.querySelectorAll<HTMLElement>('[data-document-tab-close]')].find((button) => button.getAttribute('aria-label') === `Close ${tab.getAttribute('title') ?? ''}`);
            if (!label || !close) return false;
            const labelBounds = label.getBoundingClientRect();
            const closeBounds = close.getBoundingClientRect();
            return labelBounds.right > closeBounds.left;
          }).length;
          const provenance = document.querySelector<HTMLElement>('[data-testid="source-provenance"]');
          const designHeader = document.querySelector<HTMLElement>('[data-purpose="manage-current-design"]');
          const designHeaderOverflow = designHeader ? Math.max(0, designHeader.scrollWidth - designHeader.clientWidth) : 0;
          const conversationMessages = [...document.querySelectorAll<HTMLElement>('[data-testid^="conversation-message-"]')].filter((element) => element.offsetParent !== null);
          const composer = document.querySelector<HTMLElement>('[data-slot="input-group"]');
          const transcriptViewport = document.querySelector<HTMLElement>('[data-testid="conversation-transcript-viewport"]');
          const messageComposerIntersections = composer && transcriptViewport ? conversationMessages.filter((message) => {
            const m = message.getBoundingClientRect();
            const c = composer.getBoundingClientRect();
            const v = transcriptViewport.getBoundingClientRect();
            const visibleLeft = Math.max(m.left, v.left);
            const visibleRight = Math.min(m.right, v.right);
            const visibleTop = Math.max(m.top, v.top);
            const visibleBottom = Math.min(m.bottom, v.bottom);
            return visibleLeft < visibleRight && visibleTop < visibleBottom && visibleLeft < c.right && visibleRight > c.left && visibleTop < c.bottom && visibleBottom > c.top;
          }).length : 0;
          const transcriptComposerGap = composer && transcriptViewport ? composer.getBoundingClientRect().top - transcriptViewport.getBoundingClientRect().bottom : 0;
          const sourceActions = [...document.querySelectorAll<HTMLElement>('[aria-label^="Choose pages from"], [aria-label^="Choose drawing content from"], [aria-label^="Choose model content from"]')].filter((element) => element.offsetParent !== null);
          const provenanceActionIntersections = provenance && provenance.offsetParent !== null ? sourceActions.filter((action) => {
            const a = action.getBoundingClientRect();
            const p = provenance.getBoundingClientRect();
            return a.left < p.right && a.right > p.left && a.top < p.bottom && a.bottom > p.top;
          }).length : 0;
          return { overflow: root.scrollWidth - root.clientWidth, dialogs, undersizedControls, tabCloseIntersections, provenanceActionIntersections, designHeaderOverflow, messageComposerIntersections, transcriptComposerGap, viewport: { width: innerWidth, height: innerHeight } };
        });
        expect(layout.overflow, `${name}/${sizeName} horizontal document overflow`).toBeLessThanOrEqual(1);
        expect(layout.undersizedControls, `${name}/${sizeName} controls below 24 px`).toBe(0);
        expect(layout.tabCloseIntersections, `${name}/${sizeName} tab labels intersect close controls`).toBe(0);
        expect(layout.provenanceActionIntersections, `${name}/${sizeName} provenance intersects source actions`).toBe(0);
        expect(layout.designHeaderOverflow, `${name}/${sizeName} current-design actions overflow their toolbar`).toBe(0);
        expect(layout.messageComposerIntersections, `${name}/${sizeName} messages intersect the composer`).toBe(0);
        expect(layout.transcriptComposerGap, `${name}/${sizeName} transcript viewport extends below the composer`).toBeGreaterThanOrEqual(0);
        for (const dialog of layout.dialogs) {
          expect(dialog.left).toBeGreaterThanOrEqual(0);
          expect(dialog.top).toBeGreaterThanOrEqual(0);
          expect(dialog.right).toBeLessThanOrEqual(layout.viewport.width);
          expect(dialog.bottom).toBeLessThanOrEqual(layout.viewport.height);
        }
        if (!process.env.CI) {
          await page.screenshot({ path: path.join(visualRoot, `${name}-${sizeName}.png`) });
        }
      }
      await electronApp.evaluate(({ BrowserWindow }) => BrowserWindow.getAllWindows()[0]?.setContentSize(1152, 768));
    }
    page.on('console', (message) => {
      if (message.type() === 'error' || message.type() === 'warning') {
        const text = message.text();
        if (!allowedConsoleWarnings.some((pattern) => pattern.test(text))) consoleProblems.push(`${message.type()}: ${text}`);
      }
    });
    page.on('pageerror', (error) => pageErrors.push([
      `${error.name}: ${error.message}`,
      error.stack,
    ].filter(Boolean).join('\n')));

    await page.waitForLoadState('domcontentloaded');
    expect(page.url()).toBe(`file://${path.join(appRoot, 'dist', 'index.html')}`);
    await expect.poll(() => page.evaluate(() => window.fraia.applicationMetadata?.())).toMatchObject({
      productName: 'Fraia Dev',
      userDataDirectoryName: 'Fraia Dev',
    });
    await expect(page.locator('[data-slot=menubar]')).toBeVisible();
    await expect(page.getByTestId('conversation-workspace-shell')).toBeVisible();
    await expect(page.getByTestId('empty-workspace')).toBeVisible();
    await expect.poll(async () => {
      try {
        return await page.evaluate(() => window.fraia.health());
      } catch {
        return null;
      }
    }, { timeout: 10_000 }).toMatchObject({ status: 'ok' });
    await captureVisualState('empty');
    checkpoint('first application healthy');
    await page.evaluate(() => window.fraia.aiStartOAuth({ providerId: 'openai-codex' }));
    await page.getByRole('button', { name: 'New blank model' }).first().click();
    await expect(page.getByTestId('conversation-workspace')).toBeVisible();
    await expect(page.getByTestId('blank-conversation')).toBeVisible();
    await expect(page.getByTestId('project-design-identity')).toHaveText('Untitled Project / Design 1');
    await expect(page.getByText('What would you like to design?')).toBeVisible();
    await captureVisualState('blank-conversation');
    await expect(page.getByTestId('conversation-proposal')).toHaveCount(0);
    await expect(page.getByTestId('artefact-preview')).toHaveCount(0);
    const unsavedProjectsDir = path.join(userDataDir, 'unsaved-projects');
    const createdProjectDir = fs.readdirSync(unsavedProjectsDir)
      .map((entry) => path.join(unsavedProjectsDir, entry))
      .find((entry) => fs.existsSync(path.join(entry, 'fraia.project.json'))) ?? null;
    if (!createdProjectDir) throw new Error('Fraia did not create the managed untitled project.');
    const projectManifest = JSON.parse(fs.readFileSync(path.join(createdProjectDir, 'fraia.project.json'), 'utf8'));
    const projectId = projectManifest.id;
    const designId = projectManifest.designs?.[0]?.id;
    expect(projectId, 'managed untitled project has a stable project id').toEqual(expect.any(String));
    expect(designId, 'managed untitled project has a first design').toEqual(expect.any(String));

    await electronApp.evaluate(({ dialog }, fixturePath) => {
      dialog.showOpenDialog = async (options) => options.title === 'Import project file'
        ? { canceled: false, filePaths: [fixturePath] }
        : { canceled: true, filePaths: [] };
    }, sourceFixture);
    await page.getByRole('button', { name: 'Files' }).click();
    const resourceSheet = page.getByRole('dialog', { name: 'Files and references' });
    await expect(resourceSheet.getByText('Start with a drawing or model')).toBeVisible();
    await captureVisualState('files-empty');
    await resourceSheet.getByRole('button', { name: 'Add project file' }).first().click();
    await expect(resourceSheet.getByText('architectural-drawings.pdf')).toBeVisible();
    await expect(resourceSheet.getByTestId('source-import-status')).toHaveAttribute('data-state', 'done');
    await resourceSheet.getByRole('button', { name: 'Choose pages from architectural-drawings.pdf for Design 1 references' }).click();
    const pdfBrowser = page.getByRole('dialog', { name: 'Choose a drawing area' });
    await expect(pdfBrowser.getByRole('button', { name: 'Open Page 3' })).toBeVisible();
    await captureVisualState('pdf-plan-browser');
    await pdfBrowser.getByRole('button', { name: 'Zoom out' }).click({ clickCount: 3 });
    const cropSurface = pdfBrowser.getByRole('application', { name: 'Drawing crop surface' });
    const cropBounds = await cropSurface.boundingBox();
    if (!cropBounds) throw new Error('PDF crop surface did not have measurable bounds.');
    await page.mouse.move(cropBounds.x + cropBounds.width * 0.18, cropBounds.y + cropBounds.height * 0.22);
    await page.mouse.down();
    await page.mouse.move(cropBounds.x + cropBounds.width * 0.78, cropBounds.y + cropBounds.height * 0.76, { steps: 12 });
    await page.mouse.up();
    await expect(pdfBrowser.getByTestId('pdf-view-role-inference')).toContainText('Fraia inferred');
    await expect(pdfBrowser.getByTestId('pdf-view-role-inference')).toContainText(/Page 1:.*PLAN/i);
    await pdfBrowser.getByRole('textbox', { name: 'Name' }).fill('Ground floor structural plan');
    await captureVisualState('pdf-plan-crop-metadata');
    await pdfBrowser.getByRole('button', { name: 'Close', exact: true }).first().click();
    await resourceSheet.getByRole('button', { name: 'Choose pages from architectural-drawings.pdf for Design 1 references' }).click();
    await expect(pdfBrowser.getByRole('textbox', { name: 'Name' })).toHaveValue('Ground floor structural plan');
    await pdfBrowser.getByRole('button', { name: 'Add design reference' }).click();
    await pdfBrowser.getByRole('button', { name: 'Close', exact: true }).first().click();
    await resourceSheet.getByRole('tab', { name: 'Design references' }).click();
    await expect(resourceSheet.getByText('Ground floor structural plan')).toBeVisible();
    await resourceSheet.getByRole('button', { name: 'Review interpretation for Ground floor structural plan' }).click();
    const firstInterpretation = page.getByRole('dialog', { name: "Review Fraia's interpretation" });
    await firstInterpretation.getByRole('button', { name: 'Review this reference' }).click();
    await expect(firstInterpretation.getByTestId('drawing-observation')).toContainText('unconfirmed');
    await firstInterpretation.getByRole('button', { name: 'Confirm' }).click();
    await expect(firstInterpretation.getByTestId('drawing-observation')).toContainText('confirmed');
    await captureVisualState('drawing-interpretation-plan-confirmed');
    await firstInterpretation.getByRole('button', { name: 'Done' }).click();

    await resourceSheet.getByRole('tab', { name: 'Project files' }).click();
    await electronApp.evaluate(({ dialog }, fixturePath) => {
      dialog.showOpenDialog = async (options) => options.title === 'Import project file'
        ? { canceled: false, filePaths: [fixturePath] }
        : { canceled: true, filePaths: [] };
    }, scannedPdfFixture);
    await resourceSheet.getByRole('button', { name: 'Add project file' }).first().click();
    await expect(resourceSheet.getByText('scanned-architectural-drawing.pdf')).toBeVisible();
    await resourceSheet.getByRole('button', { name: 'Choose pages from scanned-architectural-drawing.pdf for Design 1 references' }).click();
    const scannedBrowser = page.getByRole('dialog', { name: 'Choose a drawing area' });
    await scannedBrowser.getByRole('button', { name: 'Zoom out' }).click({ clickCount: 3 });
    const scannedSurface = scannedBrowser.getByRole('application', { name: 'Drawing crop surface' });
    const scannedBounds = await scannedSurface.boundingBox();
    if (!scannedBounds) throw new Error('Scanned PDF crop surface did not have measurable bounds.');
    await page.mouse.move(scannedBounds.x + scannedBounds.width * 0.12, scannedBounds.y + scannedBounds.height * 0.12);
    await page.mouse.down();
    await page.mouse.move(scannedBounds.x + scannedBounds.width * 0.88, scannedBounds.y + scannedBounds.height * 0.88, { steps: 12 });
    await page.mouse.up();
    await expect(scannedBrowser.getByTestId('pdf-view-role-inference')).toContainText('Fraia inferred', { timeout: 30_000 });
    await expect(scannedBrowser.getByTestId('pdf-view-role-inference')).toContainText(/OCR|ELEVATION/i);
    await scannedBrowser.getByTestId('pdf-view-role-inference').scrollIntoViewIfNeeded();
    await captureVisualState('scanned-ocr-inference');
    await scannedBrowser.getByRole('textbox', { name: 'Name' }).fill('User corrected scanned detail');
    await scannedBrowser.getByText('View role', { exact: true }).locator('..').getByRole('combobox').click();
    await page.getByRole('option', { name: 'Detail' }).click();
    await scannedBrowser.getByRole('button', { name: 'Close', exact: true }).first().click();
    await resourceSheet.getByRole('button', { name: 'Choose pages from scanned-architectural-drawing.pdf for Design 1 references' }).click();
    await expect(scannedBrowser.getByRole('textbox', { name: 'Name' })).toHaveValue('User corrected scanned detail');
    await expect(scannedBrowser.getByText('View role', { exact: true }).locator('..').getByRole('combobox')).toHaveText(/Detail/i);
    await scannedBrowser.getByRole('button', { name: 'Add design reference' }).click();
    await scannedBrowser.getByRole('button', { name: 'Close', exact: true }).first().click();
    await resourceSheet.getByRole('tab', { name: 'Design references' }).click();
    await resourceSheet.getByRole('button', { name: 'Review interpretation for User corrected scanned detail' }).click();
    const scannedInterpretation = page.getByRole('dialog', { name: "Review Fraia's interpretation" });
    await scannedInterpretation.getByRole('button', { name: 'Review this reference' }).click();
    await expect(scannedInterpretation.getByTestId('drawing-observation')).toContainText('unconfirmed');
    await scannedInterpretation.getByRole('button', { name: 'Confirm' }).click();
    await expect(scannedInterpretation.getByTestId('drawing-observation')).toContainText('confirmed');
    await scannedInterpretation.getByRole('button', { name: 'Done' }).click();

    await resourceSheet.getByRole('tab', { name: 'Project files' }).click();
    await electronApp.evaluate(({ dialog }, fixturePath) => {
      dialog.showOpenDialog = async (options) => options.title === 'Import project file'
        ? { canceled: false, filePaths: [fixturePath] }
        : { canceled: true, filePaths: [] };
    }, dxfFixture);
    await resourceSheet.getByRole('button', { name: 'Add project file' }).first().click();
    await expect(resourceSheet.getByText('small-frame.dxf')).toBeVisible();
    await resourceSheet.getByRole('button', { name: 'Choose drawing content from small-frame.dxf for Design 1 references' }).click();
    const dxfBrowser = page.getByRole('dialog', { name: 'Choose DXF content' });
    await expect(dxfBrowser.getByText('HIDDEN-GUIDES', { exact: true })).toBeVisible();
    await expect(dxfBrowser.getByText('Frozen').first()).toBeVisible();
    await expect(dxfBrowser.getByText('Hidden').first()).toBeVisible();
    await dxfBrowser.getByRole('checkbox', { name: 'Select layer FRAME' }).click();
    await captureVisualState('dxf-content-selection');
    await dxfBrowser.getByRole('tab', { name: 'Review' }).click();
    await dxfBrowser.getByRole('textbox', { name: 'Name' }).fill('East frame elevation');
    await dxfBrowser.getByRole('combobox', { name: 'Drawing view' }).click();
    await page.getByRole('option', { name: 'elevation' }).click();
    await dxfBrowser.getByRole('button', { name: 'Placement details' }).click();
    await dxfBrowser.getByRole('textbox', { name: 'Origin X' }).fill('2');
    await dxfBrowser.getByRole('checkbox', { name: 'I checked the drawing view and scale.' }).click();
    await captureVisualState('dxf-explicit-relation');
    await dxfBrowser.getByRole('button', { name: 'Add design reference' }).click();
    const dxfInterpretation = page.getByRole('dialog', { name: "Review Fraia's interpretation" });
    await expect(dxfInterpretation.getByTestId('drawing-observation').first()).toContainText('unconfirmed');
    await expect(dxfInterpretation.getByText('Fraia inferred').first()).toBeVisible();
    await captureVisualState('dxf-interpretation-unconfirmed');
    const firstDxfObservation = dxfInterpretation.getByTestId('drawing-observation').first();
    await firstDxfObservation.getByRole('button', { name: 'Confirm' }).click();
    await expect(firstDxfObservation).toContainText('confirmed');
    await firstDxfObservation.getByRole('button', { name: 'Alignment details' }).click();
    await firstDxfObservation.getByRole('textbox', { name: 'Origin X (m)' }).fill('2');
    await firstDxfObservation.getByRole('textbox', { name: 'Origin Y (m)' }).fill('1');
    await firstDxfObservation.getByRole('textbox', { name: 'Level Z (m)' }).fill('0');
    await firstDxfObservation.getByRole('button', { name: 'Align views' }).click();
    await expect(firstDxfObservation).toContainText('confirmed');
    const drawingReviewFailure = dxfInterpretation.getByText('Drawing review failed');
    if (await drawingReviewFailure.count()) throw new Error(`DXF reconciliation failed: ${await drawingReviewFailure.locator('..').textContent()}`);
    await dxfInterpretation.getByRole('button', { name: 'Done' }).click();
    await resourceSheet.getByRole('tab', { name: 'Design references' }).click();
    await expect(resourceSheet.getByText('East frame elevation')).toBeVisible();

    await resourceSheet.getByRole('tab', { name: 'Project files' }).click();
    await electronApp.evaluate(({ dialog }, fixturePath) => {
      dialog.showOpenDialog = async (options) => options.title === 'Import project file'
        ? { canceled: false, filePaths: [fixturePath] }
        : { canceled: true, filePaths: [] };
    }, ifcFixture);
    await resourceSheet.getByRole('button', { name: 'Add project file' }).first().click();
    await expect(resourceSheet.getByText('architect-reference.ifc')).toBeVisible();
    await resourceSheet.getByRole('button', { name: 'Choose model content from architect-reference.ifc for Design 1 references' }).click();
    const ifcBrowser = page.getByRole('dialog', { name: 'Choose IFC content' });
    await expect(ifcBrowser.getByText('Level 2')).toBeVisible();
    await expect(ifcBrowser.getByText('Elevation 3000')).toBeVisible();
    await ifcBrowser.getByRole('checkbox', { name: 'Select storey Level 2' }).click();
    await ifcBrowser.getByRole('button', { name: 'Reference details' }).click();
    await expect(ifcBrowser.getByText(/IFC4 · length unit/)).toBeVisible();
    await ifcBrowser.getByRole('textbox', { name: 'Name' }).fill('Level 2 architect reference');
    await captureVisualState('ifc-content-selection');
    await ifcBrowser.getByRole('button', { name: 'Add design reference' }).click();
    const ifcInterpretation = page.getByRole('dialog', { name: "Review Fraia's interpretation" });
    await expect(ifcInterpretation.getByText('Fraia inferred').first()).toBeVisible();
    await expect(ifcInterpretation.getByTestId('drawing-observation').first()).toContainText('unconfirmed');
    await captureVisualState('ifc-interpretation-inferred');
    await ifcInterpretation.getByRole('button', { name: 'Done' }).click();
    await resourceSheet.getByRole('tab', { name: 'Design references' }).click();
    await expect(resourceSheet.getByText('Level 2 architect reference')).toBeVisible();
    await resourceSheet.getByRole('tab', { name: 'Project files' }).click();
    await electronApp.evaluate(({ dialog }, fixturePath) => {
      dialog.showOpenDialog = async (options) => options.title === 'Import project file'
        ? { canceled: false, filePaths: [fixturePath] }
        : { canceled: true, filePaths: [] };
    }, meshFixture);
    await resourceSheet.getByRole('button', { name: 'Add project file' }).first().click();
    await expect(resourceSheet.getByText('reference-frame.obj')).toBeVisible();
    await resourceSheet.getByRole('button', { name: 'Choose model content from reference-frame.obj for Design 1 references' }).click();
    const meshBrowser = page.getByRole('dialog', { name: 'Save a 3D view' });
    await expect(meshBrowser.getByRole('application', { name: 'Reference mesh preview' })).toBeVisible();
    await meshBrowser.getByRole('button', { name: 'Reference details' }).click();
    await expect(meshBrowser.getByText(/vertices · .*triangles/)).toBeVisible();
    await meshBrowser.getByRole('textbox', { name: 'Reference name' }).fill('Architect reference mesh');
    await meshBrowser.getByRole('checkbox', { name: 'I checked this scale.' }).click();
    await meshBrowser.getByRole('button', { name: 'More options' }).click();
    await meshBrowser.getByText('Section plane').locator('..').getByRole('checkbox').click();
    await captureVisualState('mesh-reference-view');
    await meshBrowser.getByRole('button', { name: 'Add design reference' }).click();
    await resourceSheet.getByRole('tab', { name: 'Design references' }).click();
    await expect(resourceSheet.getByText('Architect reference mesh')).toBeVisible();
    const resourceAccessibility = await new AxeBuilder({ page })
      .include('[data-testid="resource-library-sheet"]')
      .setLegacyMode()
      .analyze();
    expect(resourceAccessibility.violations, 'resource Sheet axe accessibility violations').toEqual([]);
    await page.keyboard.press('Escape');
    await expect(resourceSheet).toHaveCount(0);
    checkpoint('file imported and first design references populated before Save');

    const openingRequest = 'Design a simple supported framing line for a small workshop. Keep the structure easy to inspect and analyse.';
    const typedRequest = 'Use the confirmed six metre span and simple supports from this test request.';
    await page.getByRole('textbox', { name: 'Conversation message' }).pressSequentially(openingRequest, { delay: 18 });
    await page.getByRole('button', { name: 'Send message' }).click();
    await expect(page.getByTestId('conversation-workspace').getByRole('status')).toHaveText('Fraia is working…');
    await expect(page.getByRole('button', { name: 'Cancel response' })).toBeVisible();
    await expect(page.getByTestId('conversation-workspace').getByText('Overall framing', { exact: true })).toBeVisible();
    await expect(page.getByText(openingRequest)).toBeVisible();
    const agentReplies = page.getByLabel('Fraia AI');
    await expect(agentReplies).toHaveCount(1);
    await expect(agentReplies.first()).toContainText('confirmed dimensions');
    await expect(page.getByRole('textbox', { name: 'Conversation message' })).toBeFocused();
    await expect(page.getByTestId('conversation-proposal')).toHaveCount(0);
    await expect(page.getByText('Proposed structure')).toHaveCount(0);
    const followUpRequests = [
      'The drawing in the design references is the architectural reference. What dimensions do you still need from me?',
      'Keep the design buildable and tell me which assumptions need confirmation before modelling.',
      'Do not create geometry until those project facts are clear.',
    ];
    for (const [index, request] of followUpRequests.entries()) {
      const composer = page.getByRole('textbox', { name: 'Conversation message' });
      const send = page.getByRole('button', { name: 'Send message' });
      await expect(composer).toBeEnabled();
      await expect(composer).toBeFocused();
      await composer.pressSequentially(request, { delay: 18 });
      await expect(composer).toHaveValue(request);
      await expect(send).toBeEnabled();
      await send.click();
      await expect(page.getByText(request)).toBeVisible();
      await expect(agentReplies).toHaveCount(index + 2);
      await expect(agentReplies.nth(index + 1)).toContainText('confirmed dimensions');
      await expect(composer).toBeEnabled();
      await expect(composer).toBeFocused();
      await expect(page.getByTestId('conversation-proposal')).toHaveCount(0);
    }
    const transcriptViewport = page.locator('[data-slot="message-scroller-viewport"]');
    await expect(transcriptViewport).toBeVisible();
    await transcriptViewport.evaluate((element) => {
      element.style.height = '300px';
    });
    await expect.poll(() => transcriptViewport.evaluate((element) => ({
      clientHeight: element.clientHeight,
      scrollHeight: element.scrollHeight,
    }))).toMatchObject({
      clientHeight: expect.any(Number),
      scrollHeight: expect.any(Number),
    });
    const transcriptDimensions = await transcriptViewport.evaluate((element) => ({
      clientHeight: element.clientHeight,
      scrollHeight: element.scrollHeight,
    }));
    expect(transcriptDimensions.clientHeight).toBeGreaterThan(0);
    expect(transcriptDimensions.scrollHeight).toBeGreaterThan(transcriptDimensions.clientHeight);
    await transcriptViewport.hover();
    await page.mouse.wheel(0, -1200);
    await expect.poll(() => transcriptViewport.evaluate((element) => element.scrollTop)).toBeLessThan(
      transcriptDimensions.scrollHeight - transcriptDimensions.clientHeight,
    );
    const jumpToLatest = page.getByRole('button', { name: 'Scroll to end' });
    await expect(jumpToLatest).toBeVisible();
    await jumpToLatest.click();
    await expect.poll(() => transcriptViewport.evaluate((element) => (
      element.scrollHeight - element.clientHeight - element.scrollTop
    ))).toBeLessThanOrEqual(1);
    await expect.poll(() => fs.existsSync(path.join(
      createdProjectDir,
      'designs',
      designId,
      'workspace.sqlite',
    ))).toBe(true);
    await electronApp.evaluate(({ dialog }, savedProjectDir) => {
      dialog.showSaveDialog = async () => ({ canceled: false, filePath: savedProjectDir });
    }, projectDir);
    await page.evaluate(() => window.dispatchEvent(new CustomEvent('fraia:save-project', { detail: { saveAs: false } })));
    const firstSaveDialog = page.getByRole('dialog', { name: 'Name this project and design' });
    await expect(firstSaveDialog).toBeVisible();
    await captureVisualState('first-save-dialog');
    await expect(firstSaveDialog.getByText('The folder for shared files and designs.')).toBeVisible();
    await expect(firstSaveDialog.getByText('This structural model and its conversation. Use a unique name.')).toBeVisible();
    await firstSaveDialog.getByRole('textbox', { name: 'Project name' }).fill('Workshop Project');
    await firstSaveDialog.getByRole('textbox', { name: 'Design name' }).fill('Main steel frame');
    await firstSaveDialog.getByRole('button', { name: 'Choose location' }).click();
    await expect.poll(() => fs.existsSync(path.join(projectDir, 'fraia.project.json'))).toBe(true);
    await expect.poll(() => fs.existsSync(createdProjectDir)).toBe(false);
    const savedManifest = JSON.parse(fs.readFileSync(path.join(projectDir, 'fraia.project.json'), 'utf8'));
    const savedDesignManifest = JSON.parse(fs.readFileSync(path.join(projectDir, 'designs', designId, 'fraia.design.json'), 'utf8'));
    expect(savedManifest).toMatchObject({ id: projectId, name: 'Workshop Project' });
    expect(savedManifest.designs[0]).toMatchObject({ id: designId, name: 'Main steel frame' });
    expect(savedDesignManifest).toMatchObject({ id: designId, name: 'Main steel frame' });
    await expect(page.getByRole('tab', { name: 'Main steel frame' })).toBeVisible();
    await page.getByRole('button', { name: 'New design' }).click();
    const newDesignDialog = page.getByRole('dialog', { name: 'New design' });
    await expect(newDesignDialog).toBeVisible();
    await captureVisualState('new-design-dialog');
    await newDesignDialog.getByRole('textbox', { name: 'Design name' }).fill('Lateral option');
    await newDesignDialog.getByRole('button', { name: 'Create design' }).click();
    await expect(page.getByRole('tab', { name: 'Lateral option' })).toBeVisible();
    await page.getByRole('button', { name: 'Project and design actions' }).click();
    await page.getByRole('menuitem', { name: 'Rename design' }).click();
    const renameDesignDialog = page.getByRole('dialog', { name: 'Rename design' });
    await captureVisualState('rename-design-dialog');
    await renameDesignDialog.getByRole('textbox', { name: 'Design name' }).fill('Braced option');
    await renameDesignDialog.getByRole('button', { name: 'Save name' }).click();
    await expect(page.getByRole('tab', { name: 'Braced option' })).toBeVisible();
    await expect(page.getByTestId('blank-conversation')).toBeVisible();
    await page.getByRole('button', { name: 'Files' }).click();
    await expect(page.getByRole('dialog', { name: 'Files and references' }).getByText('architectural-drawings.pdf')).toBeVisible();
    await expect(page.getByRole('dialog', { name: 'Files and references' }).getByText('scanned-architectural-drawing.pdf')).toBeVisible();
    await page.getByRole('tab', { name: 'Design references' }).click();
    await expect(page.getByText('No design references yet')).toBeVisible();
    await page.getByRole('tab', { name: 'Project files' }).click();
    await page.getByRole('button', { name: 'Choose pages from architectural-drawings.pdf for Braced option references' }).click();
    const secondPdfBrowser = page.getByRole('dialog', { name: 'Choose a drawing area' });
    await secondPdfBrowser.getByRole('button', { name: 'Open Page 2' }).click();
    await secondPdfBrowser.getByRole('button', { name: 'Zoom out' }).click({ clickCount: 3 });
    await secondPdfBrowser.getByRole('button', { name: 'Polygon' }).click();
    const secondCropSurface = secondPdfBrowser.getByRole('application', { name: 'Drawing crop surface' });
    const secondBounds = await secondCropSurface.boundingBox();
    if (!secondBounds) throw new Error('Second PDF crop surface did not have measurable bounds.');
    for (const [x, y] of [[0.25, 0.25], [0.7, 0.28], [0.66, 0.7], [0.28, 0.66]]) {
      await secondCropSurface.click({ position: { x: secondBounds.width * x, y: secondBounds.height * y } });
    }
    await secondPdfBrowser.getByRole('button', { name: 'Finish polygon' }).click();
    await secondPdfBrowser.getByRole('textbox', { name: 'Name' }).fill('North frame elevation');
    await secondPdfBrowser.getByText('View role', { exact: true }).locator('..').getByRole('combobox').click();
    await page.getByRole('option', { name: 'Elevation' }).click();
    await captureVisualState('pdf-elevation-polygon');
    await secondPdfBrowser.getByRole('button', { name: 'Add design reference' }).click();
    await secondPdfBrowser.getByRole('button', { name: 'Close', exact: true }).first().click();
    await page.getByRole('tab', { name: 'Design references' }).click();
    await expect(page.getByText('North frame elevation')).toBeVisible();
    await page.getByRole('button', { name: 'Review interpretation for North frame elevation' }).click();
    const secondInterpretation = page.getByRole('dialog', { name: "Review Fraia's interpretation" });
    await secondInterpretation.getByRole('button', { name: 'Review this reference' }).click();
    await secondInterpretation.getByRole('button', { name: 'Confirm' }).click();
    await expect(secondInterpretation.getByRole('button', { name: 'Align views' })).toBeDisabled();
    await expect(secondInterpretation.getByText('Add another confirmed view to reconcile')).toBeVisible();
    await captureVisualState('drawing-interpretation-confirmed');
    await secondInterpretation.getByRole('button', { name: 'Done' }).click();
    await page.keyboard.press('Escape');
    checkpoint('shared project file and isolated second design references verified');
    const secondRequest = 'Develop an independent lateral frame option with one diagonal brace.';
    await page.getByRole('textbox', { name: 'Conversation message' }).pressSequentially(secondRequest, { delay: 18 });
    await page.getByRole('button', { name: 'Send message' }).click();
    await expect(page.getByText(secondRequest)).toBeVisible();
    await expect(page.getByLabel('Fraia AI')).toHaveCount(1);
    await expect(page.getByLabel('Fraia AI')).toContainText('confirmed dimensions');
    await expect(page.getByTestId('conversation-proposal')).toHaveCount(0);
    await expect(page.getByText(openingRequest)).toHaveCount(0);
    const twoDesignManifest = JSON.parse(fs.readFileSync(path.join(projectDir, 'fraia.project.json'), 'utf8'));
    const secondDesignId = twoDesignManifest.designs.find((design) => design.name === 'Braced option')?.id;
    expect(secondDesignId).toEqual(expect.any(String));
    expect(secondDesignId).not.toBe(designId);
    await expect.poll(() => fs.existsSync(path.join(projectDir, 'designs', secondDesignId, 'workspace.sqlite'))).toBe(true);
    expect(path.join(projectDir, 'designs', secondDesignId, 'workspace.sqlite')).not.toBe(
      path.join(projectDir, 'designs', designId, 'workspace.sqlite'),
    );
    await page.getByRole('tab', { name: 'Main steel frame' }).click();
    await expect(page.getByText(openingRequest)).toBeVisible();
    await expect(page.getByText(secondRequest)).toHaveCount(0);
    await page.getByRole('button', { name: 'Main steel frame references' }).click();
    await expect(page.getByText('Ground floor structural plan')).toBeVisible();
    await page.keyboard.press('Escape');
    expect(fs.existsSync(path.join(userDataDir, 'conversations.sqlite'))).toBe(false);
    await expect(page.getByTestId('project-brief')).toHaveCount(0);

    await expect(page.getByRole('navigation', { name: 'Design workflow' })).toHaveCount(0);
    await expect(page.getByText('Base Model', { exact: true })).toHaveCount(0);
    await expect(page.getByText('Design Options', { exact: true })).toHaveCount(0);
    await expect(page.getByText('Analysis & Comparison', { exact: true })).toHaveCount(0);
    await transcriptViewport.evaluate((element) => { element.scrollTop = element.scrollHeight; });
    await expect(page.getByTestId('conversation-proposal')).toHaveCount(0);
    await expect(page.locator('canvas[data-fraia-canvas-role="viewport-webgl"]')).toHaveCount(0);
    await expect(page.locator('canvas[data-fraia-canvas-role="selection-overlay"]')).toHaveCount(0);
    await expect(page.getByRole('button', { name: 'Accept this direction' })).toHaveCount(0);
    checkpoint('blank conversation remained free of fabricated geometry');

    await page.getByRole('textbox', { name: 'Conversation message' }).pressSequentially(typedRequest, { delay: 18 });
    await page.getByRole('button', { name: 'Send message' }).click();
    await expect(page.getByTestId('conversation-proposal')).toBeVisible();
    await expect(page.getByText(/FRAIA_FAKE/)).toHaveCount(0);
    await page.getByTestId('conversation-proposal').scrollIntoViewIfNeeded();
    await captureVisualState('pending-proposal');
    await expect(page.getByText('Proposed structure')).toBeVisible();
    await page.getByRole('button', { name: 'Accept this direction' }).click();
    await expect(page.getByTestId('proposal-record')).toContainText('Accepted');
    await expect(page.getByRole('button', { name: 'Accept this direction' })).toHaveCount(0);
    await expect(page.getByTestId('conversation-transport-warning')).toHaveCount(0);
    await page.getByRole('button', { name: 'Run analysis' }).click();
    await expect(page.getByRole('button', { name: 'Analysing…' })).toBeVisible();
    await expect(page.locator('[data-testid="analysis-attempt"], [data-testid="conversation-transport-warning"]')).toBeVisible();
    const analysisTransportWarning = page.getByTestId('conversation-transport-warning');
    if (await analysisTransportWarning.isVisible()) {
      throw new Error(await analysisTransportWarning.innerText());
    }
    await expect(page.getByTestId('analysis-attempt')).toHaveAttribute('data-status', 'running');
    await expect(page.getByTestId('analysis-attempt')).not.toHaveAttribute('data-attempt-id', 'starting');
    await expect(page.getByRole('button', { name: 'Cancel analysis' })).toBeVisible();
    await captureVisualState('analysis-attempt-running');
    const cancelledAttemptId = await page.getByTestId('analysis-attempt').getAttribute('data-attempt-id');
    await page.getByRole('button', { name: 'Cancel analysis' }).click();
    await expect(page.getByTestId('analysis-attempt')).toHaveAttribute('data-status', 'cancelled');
    await expect(page.getByTestId('analysis-attempt')).not.toContainText(/canonical run/i);
    await captureVisualState('analysis-attempt-cancelled');
    await page.getByRole('button', { name: 'Retry analysis' }).click();
    await expect(page.getByTestId('analysis-attempt')).toHaveAttribute('data-status', 'running');
    await expect(page.getByTestId('analysis-attempt')).not.toHaveAttribute('data-attempt-id', cancelledAttemptId ?? '');
    await expect(page.getByTestId('analysis-attempt')).toHaveAttribute('data-status', 'failed');
    await expect(page.getByTestId('analysis-attempt')).toContainText('Analysis failed before Fraia could publish a result.');
    await captureVisualState('analysis-attempt-failed');
    const failedAttemptId = await page.getByTestId('analysis-attempt').getAttribute('data-attempt-id');
    await electronApp.close();
    electronApp = await electron.launch({
      args: [...deterministicLinuxRenderingArgs, '.', `--user-data-dir=${userDataDir}`],
      cwd: appRoot,
      env: {
        ...process.env,
        FRAIA_DEFAULT_PROJECT_DIR: projectDir,
        FRAIA_USER_DATA_DIR: userDataDir,
        FRAIA_DISABLE_MANAGED_CCX_BOOTSTRAP: '1',
        FRAIA_FAKE_AI_RUNTIME: '1',
        FRAIA_TEST_ANALYSIS_DELAY_MS: '2000',
      },
    });
    page = await electronApp.firstWindow();
    await page.waitForLoadState('domcontentloaded');
    await expect(page.getByTestId('empty-workspace')).toBeVisible();
    await electronApp.evaluate(({ dialog }, selectedProjectFile) => {
      dialog.showOpenDialog = async () => ({ canceled: false, filePaths: [selectedProjectFile] });
    }, path.join(projectDir, 'fraia.project.json'));
    await page.getByRole('button', { name: 'Open model' }).click();
    await expect(page.getByRole('tab', { name: 'Main steel frame' })).toBeVisible();
    const recoveredFailedAttempt = await page.evaluate(({ projectId, attemptId }) => window.fraia.analysisAttemptStatus({ projectId, attemptId }), { projectId: designId, attemptId: failedAttemptId! });
    expect(recoveredFailedAttempt.status).toBe('failed');
    await page.getByRole('button', { name: 'Run analysis' }).click();
    await expect(page.getByTestId('analysis-attempt')).toHaveAttribute('data-status', 'completed');
    const completedCanonicalRunId = await page.getByTestId('analysis-attempt').getAttribute('data-canonical-run-id');
    expect(completedCanonicalRunId).toBeTruthy();
    await expect(page.getByTestId('analysis-result-card')).toBeVisible();
    await expect(page.getByTestId('analysis-attempt')).toHaveAttribute('data-status', 'completed');
    await expect(page.getByTestId('analysis-attempt')).toContainText(/collecting|completed/);
    await expect(page.getByTestId('analysis-complete')).toBeVisible();
    await captureVisualState('analysis-result');
    await page.getByRole('button', { name: 'History' }).click();
    const analysisHistory = page.getByRole('dialog', { name: 'Analysis history' });
    await expect(analysisHistory.getByTestId('design-run-row')).toBeVisible();
    await expect(analysisHistory).not.toContainText(cancelledAttemptId!);
    await expect(analysisHistory).not.toContainText(failedAttemptId!);
    await analysisHistory.getByRole('button', { name: 'Open' }).click();
    await analysisHistory.getByRole('tab', { name: 'Run details' }).click();
    await expect(analysisHistory.getByTestId('canonical-run-details')).toContainText(completedCanonicalRunId!);
    await expect(analysisHistory.getByTestId('canonical-run-details')).toContainText('Solver:');
    await expect(analysisHistory.getByTestId('canonical-run-details')).toContainText('Resolved snapshot:');
    await captureVisualState('analysis-run-details');
    await page.keyboard.press('Escape');
    await page.getByRole('button', { name: 'Open in editor' }).last().click();
    await expect(page.getByText('Precision editor')).toBeVisible();
    await page.getByRole('button', { name: 'Record manual change' }).click();
    await expect(page.getByText('1 pending edit')).toBeVisible();
    await page.getByRole('button', { name: 'Return to conversation' }).click();
    await expect(page.getByTestId('stale-evidence')).toBeVisible();
    await captureVisualState('stale-analysis');
    await page.getByRole('button', { name: 'Rerun analysis' }).click();
    await expect(page.getByTestId('analysis-complete')).toBeVisible();
    checkpoint('typed proposal accepted once, analysed, edited, and reanalysed');

    await page.getByRole('button', { name: 'Close Braced option' }).click();
    await page.getByRole('button', { name: 'Close Main steel frame' }).click();
    await expect(page.getByTestId('empty-workspace')).toBeVisible();
    await electronApp.evaluate(({ dialog }, selectedProjectFile) => {
      dialog.showOpenDialog = async () => ({ canceled: false, filePaths: [selectedProjectFile] });
    }, path.join(projectDir, 'fraia.project.json'));
    await page.getByRole('button', { name: 'Open model' }).click();
    await expect(page.getByRole('tab', { name: 'Main steel frame' })).toBeVisible();
    await expect(page.getByRole('tab', { name: 'Braced option' })).toBeVisible();
    await page.getByRole('tab', { name: 'Braced option' }).click();
    await expect(page.getByText(secondRequest)).toBeVisible();
    await page.getByRole('tab', { name: 'Main steel frame' }).click();
    await expect(page.getByText(openingRequest)).toBeVisible();
    await expect(page.getByText(typedRequest)).toBeVisible();
    await expect(page.getByText(/FRAIA_FAKE/)).toHaveCount(0);
    await page.getByRole('button', { name: 'Files' }).click();
    await expect(page.getByRole('dialog', { name: 'Files and references' }).getByText('architectural-drawings.pdf')).toBeVisible();
    await page.getByRole('tab', { name: 'Design references' }).click();
    await expect(page.getByText('Ground floor structural plan')).toBeVisible();
    await expect(page.getByText('East frame elevation')).toBeVisible();
    await expect(page.getByText('Level 2 architect reference')).toBeVisible();
    await expect(page.getByText('Architect reference mesh')).toBeVisible();
    await expect(page.getByText('User corrected scanned detail')).toBeVisible();
    await page.getByRole('button', { name: 'Review interpretation for Ground floor structural plan' }).click();
    const reopenedPlanInterpretation = page.getByRole('dialog', { name: "Review Fraia's interpretation" });
    await expect(reopenedPlanInterpretation.getByTestId('drawing-observation')).toContainText('confirmed');
    await reopenedPlanInterpretation.getByRole('button', { name: 'Done' }).click();
    await page.getByRole('button', { name: 'Review interpretation for East frame elevation' }).click();
    const reopenedDxfInterpretation = page.getByRole('dialog', { name: "Review Fraia's interpretation" });
    await expect(reopenedDxfInterpretation.getByTestId('drawing-observation')).toHaveCount(2);
    await expect(reopenedDxfInterpretation.getByTestId('drawing-observation').first()).toContainText('confirmed');
    await expect(reopenedDxfInterpretation.getByTestId('drawing-observation').nth(1)).toContainText('unconfirmed');
    await reopenedDxfInterpretation.getByRole('button', { name: 'Done' }).click();
    await page.keyboard.press('Escape');
    await expect(page.getByRole('dialog', { name: 'Files and references' })).toHaveCount(0);
    await page.getByRole('tab', { name: 'Braced option' }).click();
    await page.getByRole('button', { name: 'Files' }).click();
    await page.getByRole('tab', { name: 'Design references' }).click();
    await expect(page.getByText('North frame elevation')).toBeVisible();
    await page.keyboard.press('Escape');
    await page.getByRole('tab', { name: 'Main steel frame' }).click();

    const accessibility = await new AxeBuilder({ page }).setLegacyMode().analyze();
    expect(pageErrors, 'unexpected renderer exceptions').toEqual([]);
    expect(consoleProblems, 'unexpected renderer warnings or errors').toEqual([]);
    expect(accessibility.violations, 'axe accessibility violations').toEqual([]);

    await electronApp.close();
    checkpoint('first application closed');
    fs.renameSync(projectDir, movedProjectDir);
    electronApp = await electron.launch({
      args: [...deterministicLinuxRenderingArgs, '.', `--user-data-dir=${userDataDir}`],
      cwd: appRoot,
      env: {
        ...process.env,
        FRAIA_DEFAULT_PROJECT_DIR: movedProjectDir,
        FRAIA_USER_DATA_DIR: userDataDir,
        FRAIA_DISABLE_MANAGED_CCX_BOOTSTRAP: '1',
        FRAIA_FAKE_AI_RUNTIME: '1',
      },
    });
    checkpoint('second application launched');
    page = await electronApp.firstWindow();
    await page.waitForLoadState('domcontentloaded');
    await expect(page.getByTestId('empty-workspace')).toBeVisible();
    await electronApp.evaluate(({ dialog }, selectedProjectFile) => {
      dialog.showOpenDialog = async () => ({ canceled: false, filePaths: [selectedProjectFile] });
    }, path.join(movedProjectDir, 'fraia.project.json'));
    await page.getByRole('button', { name: 'Open model' }).click();
    await expect(page.getByRole('tab', { name: 'Main steel frame' })).toBeVisible();
    await expect(page.getByRole('tab', { name: 'Braced option' })).toBeVisible();
    await expect(page.getByText('Workshop Project / Main steel frame')).toBeVisible();
    await expect(page.getByText(openingRequest)).toBeVisible();
    await expect(page.getByText(typedRequest)).toBeVisible();
    await expect(page.getByText(/FRAIA_FAKE/)).toHaveCount(0);
    await expect(page.getByTestId('project-brief')).toHaveCount(0);
    await expect(page.getByRole('button', { name: 'Accept this direction' })).toHaveCount(0);
    await page.getByRole('button', { name: 'Files' }).click();
    await expect(page.getByRole('dialog', { name: 'Files and references' }).getByText('architectural-drawings.pdf')).toBeVisible();
    await page.getByRole('tab', { name: 'Design references' }).click();
    await expect(page.getByText('Ground floor structural plan')).toBeVisible();
    await expect(page.getByText('East frame elevation')).toBeVisible();
    await expect(page.getByText('Level 2 architect reference')).toBeVisible();
    await expect(page.getByText('Architect reference mesh')).toBeVisible();
    await page.keyboard.press('Escape');
    await page.getByRole('tab', { name: 'Braced option' }).click();
    await page.getByRole('button', { name: 'Files' }).click();
    await page.getByRole('tab', { name: 'Design references' }).click();
    await expect(page.getByText('North frame elevation')).toBeVisible();
    await page.keyboard.press('Escape');
    checkpoint('moved project restored');
  } finally {
    await electronApp.close();
    checkpoint('final application closed');
    fs.rmSync(temporaryRoot, { recursive: true, force: true });
  }
});
