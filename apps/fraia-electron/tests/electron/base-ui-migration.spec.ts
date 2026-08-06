import { expect, test, _electron as electron } from "@playwright/test"
import type { Page } from "@playwright/test"
import AxeBuilder from "@axe-core/playwright"
import fs from "node:fs"
import os from "node:os"
import path from "node:path"

const allowedConsoleWarnings = [
  /THREE\.Clock: This module has been deprecated/,
  /Electron Security Warning \(Insecure Content-Security-Policy\)/,
]
const deterministicLinuxRenderingArgs = process.platform === "linux"
  ? ["--use-gl=angle", "--use-angle=swiftshader", "--enable-unsafe-swiftshader"]
  : []

test("desktop shell preserves keyboard and accessibility contracts", async () => {
  const appRoot = process.cwd()
  const temporaryRoot = fs.mkdtempSync(
    path.join(os.tmpdir(), "fraia-electron-e2e-"),
  )
  const projectDir = path.join(temporaryRoot, "project")
  const userDataDir = path.join(temporaryRoot, "user-data")
  fs.mkdirSync(projectDir, { recursive: true })
  fs.mkdirSync(userDataDir, { recursive: true })

  expect(
    fs.existsSync(path.join(appRoot, "dist", "index.html")),
    "Build the Electron renderer before running the desktop smoke test",
  ).toBe(true)

  const consoleProblems = [] as string[]
  const pageErrors = [] as string[]
  const electronApp = await electron.launch({
    args: [...deterministicLinuxRenderingArgs, ".", `--user-data-dir=${userDataDir}`],
    cwd: appRoot,
    env: {
      ...process.env,
      FRAIA_DEFAULT_PROJECT_DIR: projectDir,
      FRAIA_USER_DATA_DIR: userDataDir,
    },
  })

  try {
    const page = await electronApp.firstWindow()
    page.on("console", (message) => {
      if (message.type() === "error" || message.type() === "warning") {
        const text = message.text()
        if (!allowedConsoleWarnings.some((pattern) => pattern.test(text))) {
          consoleProblems.push(`${message.type()}: ${text}`)
        }
      }
    })
    page.on("pageerror", (error) => pageErrors.push(error.message))

    await page.waitForLoadState("domcontentloaded")
    await expect(page.locator("[data-slot=menubar]")).toBeVisible()
    await expect(page.locator("body")).toHaveCSS("font-size", "13px")
    await expect(page.locator('canvas[data-fraia-canvas-role="viewport-webgl"]')).toHaveCount(1)
    await expect(page.locator('canvas[data-fraia-canvas-role="selection-overlay"]')).toHaveCount(1)
    await expect(page.locator("canvas")).toHaveCount(2)

    const workflow = page.getByRole("navigation", { name: "Design workflow" })
    await expect(workflow).toBeVisible()
    await expect(workflow.locator('[aria-current="step"]')).toHaveText("Base Model")
    const gatedOptions = workflow.getByText("Design Options", { exact: true })
    await expect(gatedOptions).toHaveAttribute("aria-disabled", "true")
    await expect(workflow.getByText("Analysis & Comparison", { exact: true })).toHaveAttribute("aria-disabled", "true")
    await expect(workflow.getByRole("button", { name: "Previous" })).toHaveCount(0)
    await expect(workflow.getByRole("button", { name: "Next" })).toHaveCount(0)
    await expect(page.getByRole("button", { name: "Brief incomplete" })).toHaveCount(0)
    await gatedOptions.hover()
    await expect(page.locator('[data-slot="tooltip-content"][data-open]')).toHaveText("Generate options from the Base Model to continue.")
    await page.mouse.move(0, 0)
    await expect(page.locator('[data-slot="tooltip-content"][data-open]')).toHaveCount(0)
    await expect(workflow.locator('[aria-current="step"]')).toHaveText("Base Model")

    const firstMenuTrigger = page.locator("[data-slot=menubar-trigger]").first()
    await firstMenuTrigger.focus()
    await page.keyboard.press("ArrowDown")
    await expect(page.locator("[data-slot=menubar-content]")).toBeVisible()
    await expect(page.locator("[data-slot=menubar-item]").first()).toBeFocused()
    await page.keyboard.press("Escape")
    await expect(firstMenuTrigger).toBeFocused()

    await electronApp.evaluate(({ BrowserWindow }) => {
      BrowserWindow.getAllWindows()[0]?.setBounds({
        x: 20,
        y: 20,
        width: 900,
        height: 600,
      })
    })
    await expect(page.locator("[data-slot=menubar]")).toBeVisible()
    expect(
      await page.evaluate(() => ({
        horizontalOverflow:
          document.documentElement.scrollWidth > document.documentElement.clientWidth,
        verticalOverflow:
          document.documentElement.scrollHeight > document.documentElement.clientHeight,
      })),
      "minimum desktop bounds should not overflow the document",
    ).toEqual({ horizontalOverflow: false, verticalOverflow: false })

    const releaseNotes = Array.from(
      { length: 40 },
      (_, index) => `• Change ${index + 1} keeps the update details readable inside the dialog.`,
    ).join("\n")
    await electronApp.evaluate(({ BrowserWindow }, notes) => {
      BrowserWindow.getAllWindows()[0]?.webContents.send("fraia:updateStatus", {
        channel: "beta",
        currentVersion: "0.0.6",
        enabled: true,
        frequency: "daily",
        phase: "ready",
        releaseNotes: notes,
        trustedMetadata: true,
        version: "0.0.9",
      })
    }, releaseNotes)

    const updateDialog = page.getByRole("dialog", { name: "Fraia 0.0.9 is ready" })
    const releaseNotesRegion = page.getByRole("region", {
      name: "Release notes for Fraia 0.0.9",
    })
    await expect(updateDialog).toBeVisible()
    await expect(updateDialog.getByText("What's new")).toBeVisible()
    await expect(updateDialog.getByText("Automatic checks")).toHaveCount(0)
    await expect(updateDialog.getByText(/Fraia verifies the update/)).toHaveCount(0)
    await expect(updateDialog.getByText(/updater works independently/)).toHaveCount(0)
    await expect(releaseNotesRegion).toBeVisible()
    expect(await releaseNotesRegion.evaluate((region) => {
      const dialog = region.closest('[data-slot="dialog-content"]')
      const footer = dialog?.querySelector('[data-slot="dialog-footer"]')
      const viewport = region.querySelector('[data-slot="scroll-area-viewport"]')
      if (!(dialog instanceof HTMLElement)
        || !(footer instanceof HTMLElement)
        || !(viewport instanceof HTMLElement)) {
        throw new Error("Update dialog scroll geometry is incomplete")
      }
      const dialogBounds = dialog.getBoundingClientRect()
      const footerBounds = footer.getBoundingClientRect()
      const viewportBounds = viewport.getBoundingClientRect()
      return {
        dialogInsideWindow: dialogBounds.bottom <= window.innerHeight,
        footerBelowNotes: footerBounds.top >= viewportBounds.bottom,
        notesOverflow: viewport.scrollHeight > viewport.clientHeight,
        viewportHeight: viewport.clientHeight,
      }
    })).toEqual({
      dialogInsideWindow: true,
      footerBelowNotes: true,
      notesOverflow: true,
      viewportHeight: 176,
    })
    await page.getByRole("button", { name: "Close", exact: true }).click()
    await expect(updateDialog).toHaveCount(0)

    await page.emulateMedia({ colorScheme: "light" })
    await page.evaluate(() => localStorage.setItem("fraia:theme-mode", "dark"))
    await page.reload()
    await expect(page.locator("[data-slot=menubar]")).toBeVisible()
    await expect(page.locator('canvas[data-fraia-canvas-role="viewport-webgl"]')).toHaveCount(1)
    await expect(page.locator('canvas[data-fraia-canvas-role="selection-overlay"]')).toHaveCount(1)
    await expect(page.locator("canvas")).toHaveCount(2)
    await expect(page.locator("html")).toHaveAttribute("data-theme-mode", "system")
    await expect(page.locator("html")).not.toHaveClass(/\bdark\b/)
    await expect.poll(() => page.evaluate(() => localStorage.getItem("fraia:theme-mode"))).toBeNull()
    await page.mouse.move(0, 0)
    await expect(page.locator('[data-slot="tooltip-content"][data-open]')).toHaveCount(0)

    // Electron cannot create the blank Chromium page used by axe's partial-run
    // mode. Legacy mode runs the same rules in the application page and still
    // includes same-origin frames.
    const accessibility = await new AxeBuilder({ page }).setLegacyMode().analyze()
    expect(pageErrors, "unexpected renderer exceptions").toEqual([])
    expect(consoleProblems, "unexpected renderer warnings or errors").toEqual([])
    expect(accessibility.violations, "axe accessibility violations").toEqual([])
  } finally {
    await electronApp.close()
    fs.rmSync(temporaryRoot, { recursive: true, force: true })
  }
})

test("fake Pi runtime signs in with ChatGPT, uses Luna, completes, cancels, and reconnects", async () => {
  const appRoot = process.cwd()
  const temporaryRoot = fs.mkdtempSync(
    path.join(os.tmpdir(), "fraia-pi-electron-e2e-"),
  )
  const projectDir = path.join(temporaryRoot, "project")
  const userDataDir = path.join(temporaryRoot, "user-data")
  fs.mkdirSync(projectDir, { recursive: true })
  fs.mkdirSync(userDataDir, { recursive: true })

  const launch = (turnDelayMs = 0) => electron.launch({
    args: [...deterministicLinuxRenderingArgs, ".", `--user-data-dir=${userDataDir}`],
    cwd: appRoot,
    env: {
      ...process.env,
      FRAIA_DEFAULT_PROJECT_DIR: projectDir,
      FRAIA_FAKE_AI_RUNTIME: "1",
      FRAIA_FAKE_AI_TURN_DELAY_MS: String(turnDelayMs),
      FRAIA_USER_DATA_DIR: userDataDir,
    },
  })
  let electronApp = await launch()
  try {
    let page = await electronApp.firstWindow()
    await page.waitForLoadState("domcontentloaded")
    const applicationMenu = await electronApp.evaluate(({ Menu }) => (
      Menu.getApplicationMenu()?.items.map((item) => ({
        label: item.label,
        submenu: item.submenu?.items.map((submenuItem) => ({
          accelerator: submenuItem.accelerator?.toString() ?? null,
          role: submenuItem.role ?? null,
        })) ?? [],
      })) ?? []
    ))
    expect(applicationMenu.map((item) => item.label)).not.toContain("Developer")
    const viewMenuRoles = applicationMenu.find((item) => item.label === "View")?.submenu
      .map((item) => item.role)
    expect(viewMenuRoles).toEqual(expect.arrayContaining([
      "reload",
      "forcereload",
      "resetzoom",
      "zoomin",
      "zoomout",
      "togglefullscreen",
    ]))
    for (const role of ["reload", "forcereload", "resetzoom", "zoomin", "zoomout"]) {
      expect(applicationMenu.find((item) => item.label === "View")?.submenu
        .find((item) => item.role === role)?.accelerator).toBeTruthy()
    }
    const activateViewRole = (role: string) => electronApp.evaluate(({ BrowserWindow, Menu }, targetRole) => {
      const window = BrowserWindow.getAllWindows()[0]
      const item = Menu.getApplicationMenu()?.items
        .find((menuItem) => menuItem.label === "View")?.submenu?.items
        .find((menuItem) => menuItem.role === targetRole)
      if (!window || !item) throw new Error(`Missing View menu role: ${targetRole}`)
      item.click(undefined, window, window.webContents)
    }, role)

    expect(await electronApp.evaluate(({ BrowserWindow }) => (
      BrowserWindow.getAllWindows()[0]?.webContents.getZoomFactor()
    ))).toBe(1)
    await activateViewRole("zoomin")
    await expect.poll(() => electronApp.evaluate(({ BrowserWindow }) => (
      BrowserWindow.getAllWindows()[0]?.webContents.getZoomFactor()
    ))).toBeGreaterThan(1)
    await activateViewRole("resetzoom")
    await expect.poll(() => electronApp.evaluate(({ BrowserWindow }) => (
      BrowserWindow.getAllWindows()[0]?.webContents.getZoomFactor()
    ))).toBe(1)
    await activateViewRole("zoomout")
    await expect.poll(() => electronApp.evaluate(({ BrowserWindow }) => (
      BrowserWindow.getAllWindows()[0]?.webContents.getZoomFactor()
    ))).toBeLessThan(1)
    await activateViewRole("resetzoom")
    await expect.poll(() => electronApp.evaluate(({ BrowserWindow }) => (
      BrowserWindow.getAllWindows()[0]?.webContents.getZoomFactor()
    ))).toBe(1)

    await page.evaluate(() => document.documentElement.dataset.reloadProbe = "pending")
    let reloadComplete = page.waitForEvent("load")
    await activateViewRole("reload")
    await reloadComplete
    await expect.poll(() => page.evaluate(() => document.documentElement.dataset.reloadProbe)).toBeUndefined()
    await page.evaluate(() => document.documentElement.dataset.reloadProbe = "pending")
    reloadComplete = page.waitForEvent("load")
    await activateViewRole("forcereload")
    await reloadComplete
    await expect.poll(() => page.evaluate(() => document.documentElement.dataset.reloadProbe)).toBeUndefined()
    await expect(page.getByRole("menuitem", { name: "Developer" })).toHaveCount(0)
    await expect(page.getByRole("menuitem", { name: "Fraia AI…" })).toHaveCount(0)
    await expect(page.getByLabel(/API key/i)).toHaveCount(0)
    await expect(page.getByRole("combobox")).toHaveCount(0)
    await page.getByRole("button", { name: "Sign in required" }).click()
    await expect(page.getByRole("button", { name: "Sign out" })).toBeVisible()

    await expect(page.getByText("GPT-5.6 Luna", { exact: true }).first()).toBeVisible()
    await expect(page.getByRole("combobox")).toHaveCount(0)
    const projectPath = await page.evaluate(() => window.fraia.defaultProjectDir())
    await expect.poll(async () => page.evaluate(async ({ projectDir }) => {
      const state = await window.fraia.agentProviderStatus({ projectDir, surface: "pre_solve" })
      return `${state.selectedProviderId}/${state.selectedModelId ?? state.selectedModel}`
    }, { projectDir: projectPath })).toBe("openai-codex/gpt-5.6-luna")

    await page.getByRole("button", { name: "Start the Base Model Guide" }).click()
    await expect(page.getByText("Fake Pi response", { exact: true }).first()).toBeVisible()
    const transcript = page.getByRole("log")
    const transcriptViewport = page.getByRole("region", { name: "Messages" })
    await expect.poll(() => transcriptViewport.evaluate((element) => element.scrollTop)).toBe(0)
    await expect(transcript).toHaveAttribute("aria-busy", "false")
    await expect(page.locator('[data-slot="message"][data-author="assistant"]')).toHaveAttribute("aria-label", "Fraia AI")
    await expect(page.locator('[data-slot="message"][data-author="assistant"] [data-slot="bubble"]')).toHaveAttribute("data-variant", "ghost")
    expect(await transcriptViewport.evaluate((element) => {
      const style = getComputedStyle(element)
      return {
        scrollbarWidth: style.scrollbarWidth,
        scrollbarGutter: style.scrollbarGutter,
        usesRegistryScrollFade: element.classList.contains("scroll-fade-b"),
        hasInlineAppearanceOverrides: element.getAttribute("style") !== null,
      }
    })).toEqual({
      scrollbarWidth: "thin",
      scrollbarGutter: "stable",
      usesRegistryScrollFade: true,
      hasInlineAppearanceOverrides: false,
    })
    expect((await new AxeBuilder({ page }).setLegacyMode().analyze()).violations).toEqual([])
    await electronApp.close()

    const credentialFile = path.join(userDataDir, "ai", "credentials.bin")
    expect(fs.existsSync(credentialFile)).toBe(true)
    const encryptedCredential = fs.readFileSync(credentialFile)
    expect(encryptedCredential.includes(Buffer.from("fake-chatgpt-access-token"))).toBe(false)
    expect(encryptedCredential.includes(Buffer.from("fake-chatgpt-refresh-token"))).toBe(false)

    electronApp = await launch(5_000)
    page = await electronApp.firstWindow()
    await page.waitForLoadState("domcontentloaded")
    await expect(page.getByRole("button", { name: "Sign out" })).toBeVisible()
    await page.getByRole("button", { name: "Sign out" }).click()
    await expect(page.getByRole("button", { name: "Sign in required" })).toBeVisible()
    await page.getByRole("button", { name: "Sign in required" }).click()
    await expect(page.getByRole("button", { name: "Sign out" })).toBeVisible()

    const reply = page.getByPlaceholder("Reply to the Base Model Guide...")
    await reply.fill("Please continue the model brief.")
    await page.getByRole("button", { name: "Send", exact: true }).click()
    const cancel = page.getByRole("button", { name: "Cancel response" })
    await expect(cancel).toBeVisible()
    await expect(page.getByRole("log")).toHaveAttribute("aria-busy", "true")
    await expect(page.locator('[data-slot="marker"]')).toHaveAttribute("role", "status")
    await expect(page.locator('[data-slot="marker"]')).toContainText("Fraia AI is thinking")
    await expect(page.locator('[data-slot="message"][data-author="user"] [data-slot="bubble"]')).toHaveAttribute("data-variant", "default")
    page.once("dialog", (dialog) => void dialog.accept())
    await cancel.click()
    await expect(cancel).toHaveCount(0)
    await expect(page.getByRole("log")).toHaveAttribute("aria-busy", "false")
    await expect(page.locator('[data-slot="marker"]')).toHaveCount(0)
    await expect(page.getByRole("button", { name: "Send", exact: true })).toBeEnabled()
    await expect(reply).toHaveValue("Please continue the model brief.")
    await expect(page.getByText("Fake Pi response", { exact: true })).toHaveCount(1)
  } finally {
    await electronApp.close().catch(() => {})
    fs.rmSync(temporaryRoot, { recursive: true, force: true })
  }
})
