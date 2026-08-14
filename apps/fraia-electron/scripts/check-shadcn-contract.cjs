#!/usr/bin/env node

const fs = require("node:fs")
const path = require("node:path")
const { execFileSync } = require("node:child_process")
const ts = require("typescript")

const root = path.resolve(__dirname, "..")
const repositoryRoot = path.resolve(root, "../..")
const sourceRoot = path.join(root, "src")
const uiRoot = path.join(sourceRoot, "components/ui")
const domainUiRoot = path.join(sourceRoot, "components/domain-ui")
const findings = []
const expectedPins = {
  "@base-ui/react": "1.6.0",
  "@fontsource-variable/geist": "5.3.0",
  "@shadcn/react": "0.2.1",
  shadcn: "4.16.1",
}

function readJson(file) {
  return JSON.parse(fs.readFileSync(file, "utf8"))
}

function walk(directory) {
  return fs.readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const absolute = path.join(directory, entry.name)
    return entry.isDirectory() ? walk(absolute) : [absolute]
  })
}

function relative(file) {
  return path.relative(root, file).split(path.sep).join("/")
}

function addLineFindings(file, patterns) {
  const lines = fs.readFileSync(file, "utf8").split(/\r?\n/)
  for (const [pattern, label] of patterns) {
    lines.forEach((line, index) => {
      if (pattern.test(line)) findings.push(`${relative(file)}:${index + 1}: ${label}`)
    })
  }
}

const packageJson = readJson(path.join(root, "package.json"))
const packageLock = readJson(path.join(root, "package-lock.json"))
const componentsJson = readJson(path.join(root, "components.json"))
const globalCss = fs.readFileSync(path.join(root, "src/styles.css"), "utf8")
const skillLock = readJson(path.join(repositoryRoot, "skills-lock.json"))
const allDependencies = { ...packageJson.dependencies, ...packageJson.devDependencies }

const configChecks = [
  [componentsJson.style === "base-nova", "components.json: style must remain base-nova"],
  [componentsJson.rsc === false, "components.json: rsc must remain false"],
  [componentsJson.tsx === true, "components.json: tsx must remain true"],
  [componentsJson.iconLibrary === "lucide", "components.json: iconLibrary must remain lucide"],
  [componentsJson.tailwind?.css === "src/styles.css", "components.json: global CSS must remain src/styles.css"],
  [componentsJson.tailwind?.cssVariables === true, "components.json: semantic CSS variables must remain enabled"],
  [componentsJson.aliases?.ui === "@/components/ui", "components.json: UI alias must remain @/components/ui"],
]
for (const [passes, message] of configChecks) if (!passes) findings.push(message)
if (!globalCss.includes('@import "@fontsource-variable/geist";')) {
  findings.push("src/styles.css: official Nova Geist font import is missing")
}
if (!globalCss.includes("--font-sans: 'Geist Variable', sans-serif;")) {
  findings.push("src/styles.css: Nova Geist font token is missing")
}

for (const [dependency, expected] of Object.entries(expectedPins)) {
  if (allDependencies[dependency] !== expected) {
    findings.push(`package.json: ${dependency} must be pinned exactly to ${expected}`)
  }
  const locked = packageLock.packages?.[""]?.dependencies?.[dependency]
    ?? packageLock.packages?.[""]?.devDependencies?.[dependency]
  if (locked !== expected) findings.push(`package-lock.json: root pin for ${dependency} must be ${expected}`)
}

for (const dependency of Object.keys(allDependencies)) {
  if (dependency === "radix-ui" || dependency.startsWith("@radix-ui/")) {
    findings.push(`package.json: prohibited Radix dependency ${dependency}`)
  }
}
if (allDependencies["@fontsource-variable/inter"]) {
  findings.push("package.json: Inter font dependency is prohibited after the Nova migration")
}

const shadcnSkill = skillLock.skills?.shadcn
if (shadcnSkill?.source !== "shadcn/ui" || shadcnSkill?.sourceType !== "github") {
  findings.push("skills-lock.json: official shadcn/ui skill lock entry is missing")
}
if (!fs.existsSync(path.join(repositoryRoot, ".agents/skills/shadcn/SKILL.md"))) {
  findings.push(".agents/skills/shadcn/SKILL.md: official project-local skill is missing")
}

for (const component of [
  "alert",
  "bubble",
  "button",
  "button-group",
  "field",
  "input-group",
  "kbd",
  "marker",
  "message",
  "message-scroller",
  "select",
  "spinner",
  "tabs",
  "toggle",
  "toggle-group",
]) {
  if (!fs.existsSync(path.join(uiRoot, `${component}.tsx`))) {
    findings.push(`src/components/ui/${component}.tsx: required official component is missing`)
  }
}

const compositionFiles = walk(sourceRoot).filter((file) =>
  /\.(css|ts|tsx)$/.test(file) && !file.startsWith(`${uiRoot}${path.sep}`),
)
const forbiddenCompositionPatterns = [
  [/@base-ui\/react/, "composition must import through components/ui, not Base UI directly"],
  [/from ["']radix-ui["']|from ["']@radix-ui\//, "Radix import"],
  [/\basChild\b/, "Radix asChild API"],
  [/--radix-/, "Radix CSS variable"],
  [/data-\[state=[^\]]+\]/, "Radix state selector"],
  [/\bspace-[xy]-/, "space-x/space-y utility; use flex or grid gap"],
  [/<(?:button|input|select|textarea)\b/, "raw standard control outside the empty reviewed allowlist"],
  [/\bLoaderCircle\b/, "custom loading icon; use Spinner"],
  [/base-rhea|\bRhea\b|@fontsource-variable\/inter|Inter Variable/, "Rhea/Inter remnant after Nova migration"],
]
for (const file of compositionFiles) addLineFindings(file, forbiddenCompositionPatterns)

const domainUiExceptions = new Map([
  ["src/components/domain-ui/DocumentTabBar.tsx", {
    marker: 'data-domain-ui="document-tabs"',
    reason: "provide document closing, reordering, opening, creation, or close-focus recovery",
    test: "src/test/document-tab-bar.test.tsx",
  }],
  ["src/components/domain-ui/ResizeHandle.tsx", {
    marker: 'data-domain-ui="resize-handle"',
    reason: "The official ResizablePanelGroup must own the panel layout",
    test: "src/test/domain-resize-handle.test.tsx",
  }],
  ["src/components/domain-ui/SplitButtonSegment.tsx", {
    marker: 'data-domain-ui-exception="split-button-segment"',
    reason: "the official shadcn ButtonGroup does not expose a way",
    test: "src/test/split-button-segment.test.tsx",
  }],
])
const generatedControlVisualClassExceptions = new Map([
  ["src/components/domain-ui/DocumentTabBar.tsx", new Map([
    ["Button", "shrink-0 bg-background! hover:bg-muted!"],
    ["TabsList", "shrink-0 justify-start gap-2 rounded-none bg-background! p-0! group-data-horizontal/tabs:h-8!"],
    ["TabsTrigger", "h-8! bg-background! data-active:bg-muted! pr-8"],
  ])],
])
const actualDomainUiFiles = fs.existsSync(domainUiRoot)
  ? walk(domainUiRoot).filter((file) => /\.tsx$/.test(file)).map(relative)
  : []
for (const file of actualDomainUiFiles) {
  if (!domainUiExceptions.has(file)) findings.push(`${file}: domain UI exception is not explicitly allowlisted`)
}
for (const [file, contract] of domainUiExceptions) {
  const absolute = path.join(root, file)
  if (!fs.existsSync(absolute)) {
    findings.push(`${file}: allowlisted domain UI exception is missing`)
    continue
  }
  const text = fs.readFileSync(absolute, "utf8")
  if (!text.includes(contract.marker)) findings.push(`${file}: testable domain UI marker is missing`)
  if (!text.includes(contract.reason)) findings.push(`${file}: official-component limitation is not documented`)
  if (!fs.existsSync(path.join(root, contract.test))) findings.push(`${file}: accessibility coverage ${contract.test} is missing`)
}

const prohibitedGeneratedControlClasses = /(?:^|\s)(?:bg-|text-|font-|border(?:-|$)|rounded-|shadow-|ring-|outline-|opacity-|size-|tracking-|leading-|transition-|hover:|focus:|focus-visible:|active:|disabled:|data-\[|dark:)/
for (const file of compositionFiles.filter((candidate) => /\.tsx$/.test(candidate))) {
  const source = fs.readFileSync(file, "utf8")
  const parsed = ts.createSourceFile(file, source, ts.ScriptTarget.Latest, true, ts.ScriptKind.TSX)
  const generatedNames = new Set()
  parsed.statements.forEach((statement) => {
    if (!ts.isImportDeclaration(statement) || !ts.isStringLiteral(statement.moduleSpecifier)) return
    if (!statement.moduleSpecifier.text.startsWith("@/components/ui/")) return
    const bindings = statement.importClause?.namedBindings
    if (!bindings || !ts.isNamedImports(bindings)) return
    bindings.elements.forEach((element) => generatedNames.add(element.name.text))
  })
  if (!generatedNames.size) continue

  function stringParts(node, parts = []) {
    if (ts.isStringLiteralLike(node) || ts.isNoSubstitutionTemplateLiteral(node)) parts.push(node.text)
    node.forEachChild((child) => stringParts(child, parts))
    return parts
  }

  function visit(node) {
    if (ts.isJsxElement(node) && ts.isIdentifier(node.openingElement.tagName)) {
      const tagName = node.openingElement.tagName.text
      if (tagName === "SelectContent") {
        const hasUngroupedItem = node.children.some((child) =>
          ts.isJsxElement(child) && ts.isIdentifier(child.openingElement.tagName) && child.openingElement.tagName.text === "SelectItem"
          || ts.isJsxSelfClosingElement(child) && ts.isIdentifier(child.tagName) && child.tagName.text === "SelectItem"
        )
        if (hasUngroupedItem) {
          const line = parsed.getLineAndCharacterOfPosition(node.getStart()).line + 1
          findings.push(`${relative(file)}:${line}: SelectItem must be inside SelectGroup`)
        }
      }
      if (tagName === "InputGroup") {
        const hasRawControl = node.children.some((child) =>
          ts.isJsxElement(child) && ts.isIdentifier(child.openingElement.tagName) && ["Input", "Textarea"].includes(child.openingElement.tagName.text)
          || ts.isJsxSelfClosingElement(child) && ts.isIdentifier(child.tagName) && ["Input", "Textarea"].includes(child.tagName.text)
        )
        if (hasRawControl) {
          const line = parsed.getLineAndCharacterOfPosition(node.getStart()).line + 1
          findings.push(`${relative(file)}:${line}: InputGroup must use InputGroupInput or InputGroupTextarea`)
        }
      }
    }
    if ((ts.isJsxOpeningElement(node) || ts.isJsxSelfClosingElement(node)) && ts.isIdentifier(node.tagName) && generatedNames.has(node.tagName.text)) {
      if (node.tagName.text === "Select" && !node.attributes.properties.some((property) => ts.isJsxAttribute(property) && property.name.text === "items")) {
        const line = parsed.getLineAndCharacterOfPosition(node.getStart()).line + 1
        findings.push(`${relative(file)}:${line}: Base UI Select requires a root items prop`)
      }
      if (node.tagName.text === "SelectValue" && node.attributes.properties.some((property) => ts.isJsxAttribute(property) && property.name.text === "placeholder")) {
        const line = parsed.getLineAndCharacterOfPosition(node.getStart()).line + 1
        findings.push(`${relative(file)}:${line}: Base UI Select placeholder must be a null item, not SelectValue placeholder`)
      }
      if (node.tagName.text === "TableRow" && node.attributes.properties.some((property) => ts.isJsxAttribute(property) && property.name.text === "onClick")) {
        const line = parsed.getLineAndCharacterOfPosition(node.getStart()).line + 1
        findings.push(`${relative(file)}:${line}: interactive table rows must expose selection through an official control`)
      }
      const attribute = node.attributes.properties.find((property) => ts.isJsxAttribute(property) && property.name.text === "className")
      if (attribute?.initializer) {
        const classes = ts.isStringLiteral(attribute.initializer)
          ? attribute.initializer.text
          : ts.isJsxExpression(attribute.initializer) && attribute.initializer.expression
            ? stringParts(attribute.initializer.expression).join(" ")
            : ""
        const allowedVisualClasses = generatedControlVisualClassExceptions
          .get(relative(file))
          ?.get(node.tagName.text)
        if (classes && prohibitedGeneratedControlClasses.test(classes) && classes !== allowedVisualClasses) {
          const line = parsed.getLineAndCharacterOfPosition(node.getStart()).line + 1
          findings.push(`${relative(file)}:${line}: generated ${node.tagName.text} className must be layout-only (${classes})`)
        }
      }
      if (
        !relative(file).startsWith("src/test/")
        && ["DropdownMenuContent", "DropdownMenuSubContent", "MenubarContent", "MenubarSubContent"].includes(node.tagName.text)
      ) {
        const classes = attribute?.initializer && ts.isStringLiteral(attribute.initializer)
          ? attribute.initializer.text
          : ""
        if (!classes.includes("w-max") || !classes.includes("whitespace-nowrap")) {
          const line = parsed.getLineAndCharacterOfPosition(node.getStart()).line + 1
          findings.push(`${relative(file)}:${line}: menu content must size to its unwrapped items`)
        }
      }
    }
    ts.forEachChild(node, visit)
  }
  visit(parsed)
}

const conversationWorkspacePath = path.join(sourceRoot, "components/conversation/ConversationWorkspace.tsx")
const conversationWorkspace = fs.readFileSync(conversationWorkspacePath, "utf8")
const chatTranscriptPath = path.join(sourceRoot, "components/chat/ChatTranscript.tsx")
const chatTranscript = fs.readFileSync(chatTranscriptPath, "utf8")
for (const required of [
  "<ChatTranscript",
  "<ChatTranscriptMessage",
  "<ChatTranscriptActivity",
  "<ChatTranscriptCancel",
  "<InputGroup>",
  "<InputGroupTextarea",
  "<InputGroupAddon",
  "<Alert",
  "<Field>",
]) {
  if (!conversationWorkspace.includes(required)) findings.push(`ConversationWorkspace.tsx: conversation contract requires ${required}`)
}
for (const prohibited of [
  '@/components/ui/message-scroller',
  '@/components/ui/message',
  '@/components/ui/bubble',
  '@/components/ui/textarea',
]) {
  if (conversationWorkspace.includes(prohibited)) findings.push(`ConversationWorkspace.tsx: use the shared ChatTranscript/InputGroup composition instead of ${prohibited}`)
}
if (!conversationWorkspace.includes('aria-disabled={!composer.trim() || sending}')) {
  findings.push("ConversationWorkspace.tsx: the nested send action must use aria-disabled so InputGroup does not mute the enabled textarea")
}
if (conversationWorkspace.includes(' disabled={!composer.trim() || sending}')) {
  findings.push("ConversationWorkspace.tsx: a disabled InputGroup descendant mutes the entire composer through the official has-disabled state")
}
for (const required of [
  "autoScroll",
  "<MessageScrollerButton />",
  "messageId={messageId}",
  "scrollAnchor={scrollAnchor",
]) {
  if (!chatTranscript.includes(required)) findings.push(`ChatTranscript.tsx: official scroller contract requires ${required}`)
}

const appShellPath = path.join(sourceRoot, "components/layout/AppShell.tsx")
const appShell = fs.readFileSync(appShellPath, "utf8")
if (/<header[^>]*className=["'][^"']*\bborder-b\b/.test(appShell)) {
  findings.push("AppShell.tsx: DocumentTabBar must own the full-width divider; do not duplicate it on the header")
}
const toolbarStart = appShell.indexOf("export function ContextualWorkspaceToolbar")
const toolbarEnd = appShell.indexOf("\nfunction memberStartId", toolbarStart)
const toolbar = appShell.slice(toolbarStart, toolbarEnd)
if (toolbarStart >= 0 && toolbarEnd >= 0) {
  if (!toolbar.includes("<ToggleGroup")) findings.push("AppShell.tsx: toolbar editing modes must use ToggleGroup")
  if (!toolbar.includes("<ButtonGroup")) findings.push("AppShell.tsx: related toolbar actions must use ButtonGroup")
  if (!toolbar.includes('spacing={2}')) findings.push("AppShell.tsx: toolbar editing modes must remain separate icon controls")
  if (toolbar.includes('className="hidden xl:inline"')) findings.push("AppShell.tsx: model editing toolbar must remain icon-only")
  if (/setTimeout\s*\(|onDoubleClick/.test(toolbar)) findings.push("AppShell.tsx: toolbar must not delay opening or hide double-click behavior")
  if (/rounded-(?:none|full|[a-z0-9\[\]-]+)/.test(toolbar)) findings.push("AppShell.tsx: toolbar must not override local control radii")
  if (!appShell.includes('aria-label={`${label} settings`}\n                  aria-expanded={open}')) {
    findings.push("AppShell.tsx: split settings triggers must expose expanded state")
  }
  for (const splitControl of ["Member controls", "Snap controls", "Label controls"]) {
    if (!toolbar.includes(`aria-label="${splitControl}"`)) {
      findings.push(`AppShell.tsx: ${splitControl} must remain an official ButtonGroup composition`)
    }
  }
  for (const menuId of ["member-settings", "snap-settings", "label-settings"]) {
    if (!toolbar.includes(`'${menuId}'`)) findings.push(`AppShell.tsx: ${menuId} must remain independently addressable`)
  }
} else if (!appShell.includes("ConversationWorkspaceSurface")) {
  findings.push("AppShell.tsx: conversation-first shell must compose the typed conversation workspace")
}

for (const file of compositionFiles.filter((candidate) => /\.tsx$/.test(candidate))) {
  const lines = fs.readFileSync(file, "utf8").split(/\r?\n/)
  lines.forEach((line, index) => {
    if (line.includes("onDoubleClick")) findings.push(`${relative(file)}:${index + 1}: hidden double-click behavior`)
  })
}

const tracked = execFileSync("git", ["ls-files", "-z"], { cwd: repositoryRoot })
  .toString().split("\0").filter(Boolean)
const forbiddenTracked = tracked.filter((file) =>
  file.startsWith("apps/fraia-electron/dist/")
  || /apps\/fraia-electron\/(?:coverage|test-results|playwright-report|screenshots?|videos?|logs?)\//.test(file),
)
for (const file of forbiddenTracked) findings.push(`${file}: generated output must not be tracked`)

for (const file of tracked.filter((entry) => entry.startsWith("apps/fraia-electron/") && /\.(?:cjs|css|js|json|md|mjs|ts|tsx|yml|yaml)$/.test(entry))) {
  const absolute = path.join(repositoryRoot, file)
  if (!fs.existsSync(absolute)) continue
  const text = fs.readFileSync(absolute, "utf8")
  if (/\/Users\/[^/]+\//.test(text) || /[A-Za-z]:\\Users\\[^\\]+\\/.test(text)) {
    findings.push(`${file}: machine-specific absolute path`)
  }
}

if (findings.length) {
  console.error("Base UI Nova contract check failed:\n")
  console.error(findings.map((finding) => `- ${finding}`).join("\n"))
  process.exit(1)
}

console.log(`Base UI Nova contract check passed (${compositionFiles.length} composition files checked).`)
