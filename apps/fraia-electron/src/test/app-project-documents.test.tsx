import { render, screen, waitFor } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { beforeEach, describe, expect, it, vi } from "vitest"

import { Button } from "@/components/ui/button"
import type { WorkbenchState } from "@/lib/types"

const loadDefaultProject = vi.fn()

vi.mock("@/lib/defaultProject", async (importOriginal) => {
  const original = await importOriginal<typeof import("@/lib/defaultProject")>()
  return { ...original, loadDefaultProject }
})

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
      <Button onClick={() => props.onState({
        ...props.state,
        overview: { ...props.state.overview, fileName: "Updated project" },
      })}>Update active project</Button>
    </div>
  ),
}))

function state(projectDir: string, fileName: string): WorkbenchState {
  return {
    overview: { projectDir, fileName },
    scene: { nodes: [], members: [] },
  }
}

describe("App project documents", () => {
  beforeEach(() => {
    loadDefaultProject.mockResolvedValue(state("/projects/frame-a", "Frame A"))
    Object.assign(window, {
      fraia: {
        pickProjectFile: vi.fn().mockResolvedValue("/projects/frame-b"),
        openProject: vi.fn().mockResolvedValue(state("/projects/frame-b", "Frame B")),
        setThemeSource: vi.fn().mockResolvedValue({ ok: true }),
      },
    })
  })

  it("opens a separate project tab and updates only its active state", async () => {
    const user = userEvent.setup()
    const { default: App } = await import("@/App")
    render(<App />)

    await screen.findByTestId("active-document")
    await user.click(screen.getByRole("button", { name: "Open project" }))

    await waitFor(() => expect(screen.getByTestId("open-document-count")).toHaveTextContent("2"))
    expect(screen.getByTestId("active-document")).toHaveTextContent("/projects/frame-b")
    expect(screen.getByTestId("active-label")).toHaveTextContent("Frame B")

    await user.click(screen.getByRole("button", { name: "Update active project" }))
    expect(screen.getByTestId("active-label")).toHaveTextContent("Updated project")

    await user.click(screen.getByRole("button", { name: "Frame A" }))
    expect(screen.getByTestId("active-document")).toHaveTextContent("/projects/frame-a")
    expect(screen.getByTestId("active-label")).toHaveTextContent("Frame A")
  })
})
