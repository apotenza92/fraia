import { expect, test, _electron as electron } from "@playwright/test"
import fs from "node:fs"
import os from "node:os"
import path from "node:path"

const packagedExecutable = process.env.FRAIA_PACKAGED_EXECUTABLE
const deterministicLinuxRenderingArgs = process.platform === "linux"
  ? ["--use-gl=angle", "--use-angle=swiftshader"]
  : []

test.skip(!packagedExecutable, "run packaged verification through npm run test:package")

test("packaged app persists an edited project and exposes a deterministic solver boundary", async () => {
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
    "XDG_RUNTIME_DIR",
  ].flatMap((name) => process.env[name] ? [[name, process.env[name] as string]] : []))

  const launch = () => electron.launch({
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

  let electronApp = await launch()
  try {
    let page = await electronApp.firstWindow()
    await page.waitForLoadState("domcontentloaded")

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

    const state = await page.evaluate(async ({ projectDir }) => {
      await window.fraia.createProject({ projectDir, name: "Packaged Persistence" })
      await window.fraia.editBaseModel({
        projectDir,
        operations: [{ kind: "create_node", id: "node.packaged", x: 1, y: 2, z: 3 }],
      })
      return window.fraia.openProject({ projectDir })
    }, { projectDir })
    expect(state.state.scene.nodes).toEqual(expect.arrayContaining([
      expect.objectContaining({ id: "node.packaged" }),
    ]))

    const calculixResult = await page.evaluate(async ({ solverProjectDir }) => {
      await window.fraia.createProject({ projectDir: solverProjectDir, name: "Solver Boundary" })
      await window.fraia.seedFrameDemo({ projectDir: solverProjectDir })
      try {
        const response = await window.fraia.runFrameCalculix({ projectDir: solverProjectDir })
        return { error: null, response }
      } catch (error) {
        return { error: error instanceof Error ? error.message : String(error), response: null }
      }
    }, { solverProjectDir })
    if (requireCalculix) {
      expect(calculixResult.error).toBeNull()
      expect(calculixResult.response?.message).toContain("Saved frame CalculiX run artefacts")
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

    await page.evaluate(() => localStorage.setItem("fraia:package-smoke", "persisted"))
    await electronApp.close()

    electronApp = await launch()
    page = await electronApp.firstWindow()
    await page.waitForLoadState("domcontentloaded")
    expect(await page.evaluate(() => localStorage.getItem("fraia:package-smoke"))).toBe("persisted")
    const reopened = await page.evaluate(({ projectDir }) => window.fraia.openProject({ projectDir }), { projectDir })
    expect(reopened.state.scene.nodes).toEqual(expect.arrayContaining([
      expect.objectContaining({ id: "node.packaged" }),
    ]))
  } finally {
    await electronApp.close().catch(() => {})
    fs.rmSync(temporaryRoot, { recursive: true, force: true })
  }
})
