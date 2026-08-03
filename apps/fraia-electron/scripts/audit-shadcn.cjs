#!/usr/bin/env node

const fs = require("node:fs")
const path = require("node:path")
const crypto = require("node:crypto")
const { spawnSync } = require("node:child_process")

const root = path.resolve(__dirname, "..")
const cli = path.join(root, "node_modules/.bin", process.platform === "win32" ? "shadcn.cmd" : "shadcn")
const expected = {
  cli: "4.16.1",
  baseUi: "1.6.0",
  shadcnReact: "0.2.1",
  geist: "5.3.0",
  style: "base-nova",
  base: "base",
  iconLibrary: "lucide",
  preset: "b2fA",
  presetValues: {
    style: "nova",
    baseColor: "neutral",
    theme: "neutral",
    chartColor: "neutral",
    iconLibrary: "lucide",
    font: "geist",
    fontHeading: "inherit",
    radius: "default",
    menuAccent: "subtle",
    menuColor: "default",
  },
}

function run(args) {
  const result = spawnSync(cli, args, {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 32 * 1024 * 1024,
  })
  if (result.status !== 0) {
    process.stderr.write(result.stdout || "")
    process.stderr.write(result.stderr || "")
    process.exit(result.status || 1)
  }
  return result.stdout
}

const packageJson = JSON.parse(fs.readFileSync(path.join(root, "package.json"), "utf8"))
const pins = { ...packageJson.dependencies, ...packageJson.devDependencies }
const failures = []

const version = run(["--version"]).trim()
if (version !== expected.cli || pins.shadcn !== expected.cli) {
  failures.push(`shadcn CLI must be pinned and resolved to ${expected.cli}; found pin ${pins.shadcn ?? "missing"}, CLI ${version}`)
}
if (pins["@base-ui/react"] !== expected.baseUi) {
  failures.push(`@base-ui/react must be pinned exactly to ${expected.baseUi}; found ${pins["@base-ui/react"] ?? "missing"}`)
}
if (pins["@shadcn/react"] !== expected.shadcnReact) {
  failures.push(`@shadcn/react must be pinned exactly to ${expected.shadcnReact}; found ${pins["@shadcn/react"] ?? "missing"}`)
}
if (pins["@fontsource-variable/geist"] !== expected.geist) {
  failures.push(`@fontsource-variable/geist must be pinned exactly to ${expected.geist}; found ${pins["@fontsource-variable/geist"] ?? "missing"}`)
}
if (pins["@fontsource-variable/inter"]) {
  failures.push("@fontsource-variable/inter must not remain after the Nova migration")
}

const info = JSON.parse(run(["info", "--json"]))
for (const [actual, wanted, label] of [
  [info.config?.style, expected.style, "style"],
  [info.config?.base, expected.base, "primitive base"],
  [info.config?.iconLibrary, expected.iconLibrary, "icon library"],
  [info.preset?.code, expected.preset, "resolved preset"],
  [info.project?.tailwindVersion, "v4", "Tailwind major"],
  [info.project?.rsc, false, "RSC setting"],
  [info.project?.typescript, true, "TypeScript setting"],
]) {
  if (actual !== wanted) failures.push(`${label}: expected ${wanted}, found ${actual}`)
}
for (const [key, wanted] of Object.entries(expected.presetValues)) {
  const actual = info.preset?.values?.[key]
  if (actual !== wanted) failures.push(`preset ${key}: expected ${wanted}, found ${actual}`)
}

const components = [...new Set(info.components ?? [])].sort()
const drift = []
const canonicalTargets = new Set()
for (const component of components) {
  const viewed = JSON.parse(run(["view", component]))
  const registryItem = viewed.find((item) => item.name === component)
  const canonicalFiles = [...new Set((registryItem?.files ?? [])
    .filter((file) => file.type === "registry:ui")
    .map((file) => path.basename(file.path)))]
  if (!canonicalFiles.length) {
    failures.push(`${component}: official registry metadata exposed no canonical UI files`)
    continue
  }
  for (const canonicalFile of canonicalFiles) {
    const target = path.join(root, "src/components/ui", canonicalFile)
    canonicalTargets.add(target)
    const before = fs.existsSync(target)
      ? crypto.createHash("sha256").update(fs.readFileSync(target)).digest("hex")
      : null
    const dryRun = run(["add", component, "--dry-run"])
    const afterDryRun = fs.existsSync(target)
      ? crypto.createHash("sha256").update(fs.readFileSync(target)).digest("hex")
      : null
    if (before !== afterDryRun) failures.push(`${path.relative(root, target)}: dry run wrote to a canonical file`)
    if (!dryRun.includes("dry run")) failures.push(`${component}: CLI did not confirm dry-run mode`)
    const output = run(["add", component, "--diff", canonicalFile])
    const after = fs.existsSync(target)
      ? crypto.createHash("sha256").update(fs.readFileSync(target)).digest("hex")
      : null
    if (before !== after) failures.push(`${path.relative(root, target)}: audit command wrote to a canonical file`)
    if (!output.includes("No changes.")) {
      const action = output.includes("(create)") ? "create" : output.includes("(overwrite)") ? "overwrite" : "change"
      drift.push({ component, file: `src/components/ui/${canonicalFile}`, action, output })
    }
  }
}

if (drift.length) {
  failures.push(...drift.map(({ file, action }) => `${file}: registry would ${action}`))
}

console.log(`Resolved shadcn ${version}; audited ${components.length} components and ${canonicalTargets.size} canonical files against ${expected.style} preset ${expected.preset}.`)
if (failures.length) {
  console.error("shadcn registry audit failed:\n")
  console.error(failures.map((failure) => `- ${failure}`).join("\n"))
  for (const item of drift) {
    console.error(`\n--- ${item.component} diff ---\n${item.output.trim()}`)
  }
  process.exit(1)
}

console.log("All canonical generated files are aligned. No project files were written.")
