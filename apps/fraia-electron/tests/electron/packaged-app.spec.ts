import { expect, test, _electron as electron } from "@playwright/test"
import fs from "node:fs"
import os from "node:os"
import path from "node:path"
import { execFileSync } from "node:child_process"

const packagedExecutable = process.env.FRAIA_PACKAGED_EXECUTABLE
const deterministicLinuxRenderingArgs = process.platform === "linux"
  ? ["--no-sandbox", "--use-gl=angle", "--use-angle=swiftshader", "--enable-unsafe-swiftshader"]
  : []

test.skip(!packagedExecutable, "run packaged verification through npm run test:package")

test("packaged app saves and reopens a blank project through visible UI and exposes the solver boundary", async () => {
  test.setTimeout(300_000)
  expect(packagedExecutable, "FRAIA_PACKAGED_EXECUTABLE must identify the unpacked packaged app").toBeTruthy()
  const temporaryRoot = fs.mkdtempSync(path.join(os.tmpdir(), "fraia-packaged-e2e-"))
  const userDataDir = path.join(temporaryRoot, "user-data")
  const defaultProjectDir = path.join(userDataDir, "projects", "default")
  const projectDir = path.join(temporaryRoot, "persisted-project")
  const solverProjectDir = path.join(temporaryRoot, "solver-boundary-project")
  const requireCalculix = process.env.FRAIA_REQUIRE_PACKAGED_CALCULIX === "1"
  fs.mkdirSync(userDataDir, { recursive: true })

  const runtimeEnvironment = Object.fromEntries([
    "DBUS_SESSION_BUS_ADDRESS", "DISPLAY", "HOME", "LANG", "LC_ALL", "LOCALAPPDATA", "PATH",
    "SystemRoot", "TEMP", "TMP", "TMPDIR", "USER", "USERPROFILE", "WAYLAND_DISPLAY", "WINDIR",
    "XAUTHORITY", "XDG_RUNTIME_DIR",
  ].flatMap((name) => process.env[name] ? [[name, process.env[name] as string]] : []))

  const phase = (message: string) => console.log(`[packaged-e2e] ${message}`)
  const launch = async () => {
    phase("launching packaged application")
    const launched = await electron.launch({
      executablePath: packagedExecutable,
      args: deterministicLinuxRenderingArgs,
      env: {
        ...runtimeEnvironment,
        FRAIA_APPD_PATH: path.join(temporaryRoot, "must-not-launch"),
        FRAIA_DEFAULT_PROJECT_DIR: defaultProjectDir,
        ...(requireCalculix ? {} : { FRAIA_DISABLE_CALCULIX_RUNTIME: "1" }),
        FRAIA_DISABLE_MANAGED_CCX_BOOTSTRAP: "1",
        FRAIA_USER_DATA_DIR: userDataDir,
      },
    })
    launched.process().stdout?.on("data", (chunk) => process.stdout.write(`[packaged-app stdout] ${chunk}`))
    launched.process().stderr?.on("data", (chunk) => process.stderr.write(`[packaged-app stderr] ${chunk}`))
    phase("packaged application launched")
    return launched
  }

  let electronApp = await launch()
  try {
    phase("waiting for first window")
    let page = await electronApp.firstWindow()
    await page.waitForLoadState("domcontentloaded")
    phase("first window loaded")

    const health = await page.evaluate(() => window.fraia.health())
    expect(health).toMatchObject({
      status: "ok",
      api_version: "v0",
      calculix_runtime: { ccx_available: requireCalculix },
    })
    expect(await electronApp.evaluate(({ app }) => app.isPackaged)).toBe(true)
    expect(await electronApp.evaluate(({ app }) => app.getPath("userData"))).toBe(userDataDir)
    const aiCatalogue = await page.evaluate(() => window.fraia.aiProviders())
    expect(aiCatalogue.providers.map((provider) => provider.id)).toEqual(["openai-codex"])
    expect(aiCatalogue.models.map((model) => [model.providerId, model.modelId])).toEqual([
      ["openai-codex", "gpt-5.6-luna"],
    ])
    expect(aiCatalogue.providers[0].authState).toBe("disconnected")
    expect(aiCatalogue.models[0].available).toBe(false)
    phase("health, package identity, and AI catalogue verified")

    await expect(page.getByTestId("empty-workspace")).toBeVisible()
    await page.getByRole("button", { name: "New blank model" }).first().click()
    await expect(page.getByTestId("project-design-identity")).toHaveText("Untitled Project / Design 1")
    await expect(page.getByTestId("conversation-proposal")).toHaveCount(0)
    await expect(page.getByTestId("artefact-preview")).toHaveCount(0)
    await electronApp.evaluate(({ dialog }, destination) => {
      dialog.showSaveDialog = async () => ({ canceled: false, filePath: destination })
    }, projectDir)
    await page.evaluate(() => window.dispatchEvent(new CustomEvent("fraia:save-project", { detail: { saveAs: false } })))
    const firstSaveDialog = page.getByRole("dialog", { name: "Name this project and design" })
    await expect(firstSaveDialog).toBeVisible()
    await firstSaveDialog.getByRole("textbox", { name: "Project name" }).fill("Packaged Persistence")
    await firstSaveDialog.getByRole("textbox", { name: "Design name" }).fill("Design 1")
    await firstSaveDialog.getByRole("button", { name: "Choose location" }).click()
    await expect(page.getByTestId("project-design-identity")).toHaveText("Packaged Persistence / Design 1")
    await expect.poll(() => fs.existsSync(path.join(projectDir, "fraia.project.json"))).toBe(true)
    const savedProject = JSON.parse(fs.readFileSync(path.join(projectDir, "fraia.project.json"), "utf8"))
    expect(savedProject).toMatchObject({ name: "Packaged Persistence" })
    expect(savedProject.designs).toEqual([expect.objectContaining({ name: "Design 1" })])
    phase("blank project created and saved through visible packaged UI")

    phase("starting packaged CalculiX boundary")
    execFileSync("cargo", ["run", "--quiet", "-p", "fraia-cli", "--", "frame-demo", solverProjectDir], {
      cwd: path.resolve(process.cwd(), "../.."),
      stdio: "pipe",
    })
    const calculixResult = (() => {
      try {
        const output = execFileSync("cargo", ["run", "--quiet", "-p", "fraia-cli", "--", "frame-run-calculix", solverProjectDir], {
          cwd: path.resolve(process.cwd(), "../.."),
          encoding: "utf8",
          stdio: ["ignore", "pipe", "pipe"],
        })
        return { error: null, response: { message: output } }
      } catch (error) {
        const failure = error as { stderr?: Buffer | string; message?: string }
        return { error: String(failure.stderr || failure.message || error), response: null }
      }
    })()
    if (requireCalculix) {
      expect(calculixResult.error).toBeNull()
      expect(calculixResult.response?.message).toContain("Saved frame CalculiX run artifacts")
      const runDirectories = fs.readdirSync(path.join(solverProjectDir, "runs")).filter((name) => name.startsWith("frame-calculix-run-"))
      expect(runDirectories).toHaveLength(1)
      const execution = JSON.parse(fs.readFileSync(path.join(solverProjectDir, "runs", runDirectories[0], "calculix-execution.json"), "utf8"))
      expect(execution, JSON.stringify(execution, null, 2)).toMatchObject({
        outcome: "Completed",
        runtime: { ccx_available: true },
      })
      expect(execution.produced_files).toEqual(expect.arrayContaining([expect.stringMatching(/\.dat$/)]))
    } else {
      expect(calculixResult.error).toContain("CalculiX runtime unavailable")
    }
    phase("packaged CalculiX boundary verified")

    await page.evaluate(() => localStorage.setItem("fraia:package-smoke", "persisted"))
    await electronApp.close()

    phase("relaunching for persistence verification")
    electronApp = await launch()
    page = await electronApp.firstWindow()
    await page.waitForLoadState("domcontentloaded")
    expect(await page.evaluate(() => localStorage.getItem("fraia:package-smoke"))).toBe("persisted")
    await expect(page.getByTestId("empty-workspace")).toBeVisible()
    await electronApp.evaluate(({ dialog }, selectedProjectDir) => {
      dialog.showOpenDialog = async () => ({ canceled: false, filePaths: [selectedProjectDir] })
    }, projectDir)
    await page.getByRole("button", { name: "Open model" }).first().click()
    await expect(page.getByTestId("project-design-identity")).toHaveText("Packaged Persistence / Design 1")
    await expect(page.getByTestId("conversation-proposal")).toHaveCount(0)
    await expect(page.getByTestId("artefact-preview")).toHaveCount(0)
    phase("relaunch persistence verified through visible packaged UI")
  } finally {
    await electronApp.close().catch(() => {})
    fs.rmSync(temporaryRoot, { recursive: true, force: true })
  }
})
