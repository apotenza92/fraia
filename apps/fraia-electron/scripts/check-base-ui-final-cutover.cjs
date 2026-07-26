#!/usr/bin/env node

const fs = require("node:fs")
const path = require("node:path")

const root = path.resolve(__dirname, "..")
const sourceRoot = path.join(root, "src")
const findings = []

// shadcn's official base-rhea ToggleGroup currently emits this redundant
// compatibility selector. Base UI exposes aria-pressed/data-pressed at runtime;
// keep the generated primitive untouched while rejecting the selector elsewhere.
function isOfficialToggleGroupCompatibilitySelector(relative, line, label) {
  return label === "Radix state selector"
    && relative === "src/components/ui/toggle-group.tsx"
    && line.includes("data-[state=on]:bg-muted")
}

const forbiddenSourcePatterns = [
  [/from ["']radix-ui["']|from ["']@radix-ui\//, "Radix import"],
  [/\basChild\b/, "Radix asChild prop"],
  [/--radix-/, "Radix CSS variable"],
  [
    /data-\[state=(?:open|closed|checked|unchecked|active|inactive|on|off|indeterminate)\]/,
    "Radix state selector",
  ],
]

function walk(directory) {
  for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
    const absolute = path.join(directory, entry.name)
    if (entry.isDirectory()) {
      walk(absolute)
      continue
    }
    if (!/\.(css|ts|tsx)$/.test(entry.name)) continue

    const relative = path.relative(root, absolute)
    const lines = fs.readFileSync(absolute, "utf8").split(/\r?\n/)
    for (const [pattern, label] of forbiddenSourcePatterns) {
      lines.forEach((line, index) => {
        if (pattern.test(line) && !isOfficialToggleGroupCompatibilitySelector(relative, line, label)) {
          findings.push(`${relative}:${index + 1}: ${label}`)
        }
      })
    }
  }
}

walk(sourceRoot)

const packageJson = JSON.parse(
  fs.readFileSync(path.join(root, "package.json"), "utf8"),
)
const componentsJson = JSON.parse(
  fs.readFileSync(path.join(root, "components.json"), "utf8"),
)
const allDependencies = {
  ...packageJson.dependencies,
  ...packageJson.devDependencies,
}

if (!allDependencies["@base-ui/react"]) {
  findings.push("package.json: @base-ui/react dependency is missing")
}
if (componentsJson.style !== "base-rhea") {
  findings.push(
    `components.json: expected style base-rhea, found ${componentsJson.style ?? "missing"}`,
  )
}
for (const dependency of Object.keys(allDependencies)) {
  if (dependency === "radix-ui" || dependency.startsWith("@radix-ui/")) {
    findings.push(`package.json: obsolete dependency ${dependency}`)
  }
}

if (findings.length) {
  console.error("Base UI final-cutover guard failed:\n")
  console.error(findings.map((finding) => `- ${finding}`).join("\n"))
  console.error(
    "\nRun this guard only after the staged migration is ready for final cutover.",
  )
  process.exit(1)
}

console.log("Base UI final-cutover guard passed.")
