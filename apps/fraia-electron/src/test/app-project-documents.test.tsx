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
    Object.assign(window, {
      fraia: {
        pickProjectFile: vi.fn()
          .mockResolvedValueOnce("/projects/frame-a")
          .mockResolvedValueOnce("/projects/frame-b"),
        openProject: vi.fn()
          .mockResolvedValueOnce(state("/projects/frame-a", "Frame A"))
          .mockResolvedValueOnce(state("/projects/frame-b", "Frame B")),
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
    expect(screen.getByTestId("active-document")).toHaveTextContent("/projects/frame-b")
    expect(screen.getByTestId("active-label")).toHaveTextContent("Frame B")

    await user.click(screen.getByRole("button", { name: "Update active project" }))
    expect(screen.getByTestId("active-label")).toHaveTextContent("Updated project")

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
  })
})
