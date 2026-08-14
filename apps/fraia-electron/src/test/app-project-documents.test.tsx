import { render, screen, waitFor } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { beforeEach, describe, expect, it, vi } from "vitest"

import { Button } from "@/components/ui/button"
import type { WorkbenchState } from "@/lib/types"

vi.mock("@/lib/theme", () => ({
  useSystemTheme: vi.fn(),
}))

vi.mock("@/components/layout/AppShell", () => ({
  AppShell: (props: any) => (
    <div>
      <span data-testid="active-document">{props.activeDocumentId}</span>
      <span data-testid="active-label">{props.documentTabs.find((tab: any) => tab.id === props.activeDocumentId)?.label}</span>
      <span data-testid="open-document-count">{props.documentTabs.length}</span>
      {props.documentTabs.map((tab: any) => (
        <Button key={tab.id} onClick={() => props.onDocumentSelect(tab.id)}>{tab.label}</Button>
      ))}
      <Button onClick={props.onOpenDocument}>Open project</Button>
      {props.documentTabs.map((tab: any) => (
        <Button key={`close-${tab.id}`} onClick={() => props.onDocumentClose(tab.id)}>Close {tab.label}</Button>
      ))}
      <Button onClick={() => props.onState({
        ...props.state,
        overview: { ...props.state.overview, designName: "Updated design" },
      })}>Update active project</Button>
      <Button onClick={() => window.dispatchEvent(new CustomEvent("fraia:save-project"))}>Save project</Button>
      <Button onClick={props.onNewBlankModel}>New design</Button>
      <Button onClick={props.onRenameDesign}>Rename design</Button>
    </div>
  ),
}))

function state(projectDir: string, designName: string, managedUnsaved = false): WorkbenchState {
  const slug = projectDir.split("/").pop() ?? "design"
  return {
    overview: {
      projectDir,
      projectId: `project-${slug}`,
      projectName: managedUnsaved ? "Untitled Project" : `${designName} Project`,
      designId: `design-${slug}`,
      designName,
      documentId: `design-${slug}`,
      managedUnsaved,
    },
    scene: { nodes: [], members: [] },
  }
}

describe("App project documents", () => {
  beforeEach(() => {
    Object.assign(window, {
      fraia: {
        pickProjectFile: vi.fn()
          .mockResolvedValueOnce("/projects/frame-a")
          .mockResolvedValueOnce("/projects/frame-b"),
        openProject: vi.fn()
          .mockResolvedValueOnce(state("/projects/frame-a", "Frame A"))
          .mockResolvedValueOnce(state("/projects/frame-b", "Frame B")),
        createUntitledProject: vi.fn().mockResolvedValue("/managed/unsaved/untitled-1"),
        createProject: vi.fn().mockResolvedValue(state("/managed/unsaved/untitled-1", "Design 1", true)),
        saveProject: vi.fn().mockImplementation(async (payload) => ({
          ...state("/projects/saved-frame", payload.designName ?? "Design 1"),
          overview: {
            ...state("/projects/saved-frame", payload.designName ?? "Design 1").overview,
            projectId: payload.projectId,
            projectName: payload.projectName,
            designId: payload.designId,
            documentId: payload.designId,
            designName: payload.designName,
            projectRootDir: "/projects/saved-frame",
          },
        })),
        createDesign: vi.fn().mockResolvedValue({
          ...state("/projects/frame-a/designs/design-option", "Option B"),
          overview: {
            ...state("/projects/frame-a/designs/design-option", "Option B").overview,
            projectId: "project-frame-a",
            projectName: "Frame A Project",
            projectRootDir: "/projects/frame-a",
          },
        }),
        renameDesign: vi.fn(),
        deleteDesign: vi.fn(),
        conversationCancelDesign: vi.fn().mockResolvedValue({ cancelled: 1 }),
        setThemeSource: vi.fn().mockResolvedValue({ ok: true }),
      },
    })
  })

  it("opens a separate project tab and updates only its active state", async () => {
    const user = userEvent.setup()
    const { default: App } = await import("@/App")
    render(<App />)

    await user.click(screen.getByRole("button", { name: "Open model" }))
    await user.click(await screen.findByRole("button", { name: "Open project" }))
    await waitFor(() => expect(screen.getByTestId("open-document-count")).toHaveTextContent("2"))
    expect(screen.getByTestId("active-document")).toHaveTextContent("design-frame-b")
    expect(screen.getByTestId("active-label")).toHaveTextContent("Frame B")

    await user.click(screen.getByRole("button", { name: "Update active project" }))
    expect(screen.getByTestId("active-label")).toHaveTextContent("Frame B")

  })

  it("starts empty and can close the final document", async () => {
    const user = userEvent.setup()
    const { default: App } = await import("@/App")
    render(<App />)

    expect(screen.getByTestId("empty-workspace")).toBeVisible()
    expect(screen.getByText("No models are open")).toBeVisible()

    await user.click(screen.getByRole("button", { name: "Open model" }))
    await screen.findByTestId("active-document")
    await user.click(screen.getByRole("button", { name: "Close Frame A" }))

    expect(screen.getByTestId("empty-workspace")).toBeVisible()
    expect(window.fraia.conversationCancelDesign).toHaveBeenCalledWith({ designId: "design-frame-a" })
  })

  it("creates an untitled model without opening a location picker", async () => {
    const user = userEvent.setup()
    const { default: App } = await import("@/App")
    render(<App />)

    const newModelButtons = screen.getAllByRole("button", { name: "New blank model" })
    await user.click(newModelButtons[newModelButtons.length - 1])

    await waitFor(() => expect(window.fraia.createUntitledProject).toHaveBeenCalledOnce())
    expect(window.fraia.createProject).toHaveBeenCalledWith({
      projectDir: "/managed/unsaved/untitled-1",
      name: "Untitled Project",
    })
    expect(screen.getByTestId("active-label")).toHaveTextContent("Design 1")
  })

  it("collects the two required names before the first durable save", async () => {
    const user = userEvent.setup()
    const { default: App } = await import("@/App")
    render(<App />)

    const newModelButtons = screen.getAllByRole("button", { name: "New blank model" })
    await user.click(newModelButtons[newModelButtons.length - 1])
    await user.click(await screen.findByRole("button", { name: "Save project" }))

    const projectInput = screen.getByRole("textbox", { name: "Project name" })
    const designInput = screen.getByRole("textbox", { name: "Design name" })
    expect(projectInput).toHaveValue("Untitled Project")
    expect(designInput).toHaveValue("Design 1")

    await user.clear(projectInput)
    await user.clear(designInput)
    await user.click(screen.getByRole("button", { name: "Choose location" }))
    expect(projectInput).toHaveFocus()
    expect(screen.getByText("Enter a project name.")).toBeVisible()
    expect(screen.getByText("Enter a design name.")).toBeVisible()

    await user.type(projectInput, "Workshop")
    await user.type(designInput, "Main steel frame")
    await user.click(screen.getByRole("button", { name: "Choose location" }))

    await waitFor(() => expect(window.fraia.saveProject).toHaveBeenCalledWith({
      projectDir: "/managed/unsaved/untitled-1",
      projectId: "project-untitled-1",
      designId: "design-untitled-1",
      designIds: ["design-untitled-1"],
      projectName: "Workshop",
      designName: "Main steel frame",
      suggestedName: "Workshop",
      saveAs: false,
    }))
    expect(screen.getByTestId("active-label")).toHaveTextContent("Main steel frame")
  })

  it("creates a named design in the active project and keys its tab by design id", async () => {
    const user = userEvent.setup()
    const { default: App } = await import("@/App")
    render(<App />)

    await user.click(screen.getByRole("button", { name: "Open model" }))
    await user.click(await screen.findByRole("button", { name: "New design" }))
    const name = screen.getByRole("textbox", { name: "Design name" })
    await user.click(screen.getByRole("button", { name: "Create design" }))
    expect(name).toHaveFocus()
    expect(screen.getByText("Enter a design name.")).toBeVisible()
    await user.type(name, "Option B")
    await user.click(screen.getByRole("button", { name: "Create design" }))

    await waitFor(() => expect(window.fraia.createDesign).toHaveBeenCalledWith({
      projectDir: "/projects/frame-a",
      projectId: "project-frame-a",
      designName: "Option B",
    }))
    expect(screen.getByTestId("active-document")).toHaveTextContent("design-design-option")
    expect(screen.getByTestId("active-label")).toHaveTextContent("Option B")
    expect(screen.getByTestId("open-document-count")).toHaveTextContent("2")
  })
})
