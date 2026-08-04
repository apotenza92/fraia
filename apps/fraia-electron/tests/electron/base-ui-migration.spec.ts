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

    await page.evaluate(() => localStorage.setItem("fraia:theme-mode", "dark"))
    await page.reload()
    await expect(page.locator("[data-slot=menubar]")).toBeVisible()
    await expect(page.locator('canvas[data-fraia-canvas-role="viewport-webgl"]')).toHaveCount(1)
    await expect(page.locator('canvas[data-fraia-canvas-role="selection-overlay"]')).toHaveCount(1)
    await expect(page.locator("canvas")).toHaveCount(2)
    await expect(page.locator("html")).toHaveAttribute("data-theme-mode", "system")
    await expect.poll(() => page.evaluate(() => localStorage.getItem("fraia:theme-mode"))).toBeNull()

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
    expect(await electronApp.evaluate(({ Menu }) => (
      Menu.getApplicationMenu()?.items.map((item) => item.label) ?? []
    ))).not.toContain("Developer")
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
    await expect(transcript).toHaveAttribute("aria-busy", "false")
    await expect(page.locator('[data-slot="message"][data-author="assistant"]')).toHaveAttribute("aria-label", "Fraia AI")
    await expect(page.locator('[data-slot="message"][data-author="assistant"] [data-slot="bubble"]')).toHaveAttribute("data-variant", "ghost")
    expect(await transcriptViewport.evaluate((element) => {
      const style = getComputedStyle(element)
      return {
        scrollbarWidth: style.scrollbarWidth,
        scrollbarColor: style.scrollbarColor,
        scrollbarGutter: style.scrollbarGutter,
        maskImage: style.maskImage,
        webkitMaskImage: style.webkitMaskImage,
      }
    })).toEqual({
      scrollbarWidth: "auto",
      scrollbarColor: "auto",
      scrollbarGutter: "auto",
      maskImage: "none",
      webkitMaskImage: "none",
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
