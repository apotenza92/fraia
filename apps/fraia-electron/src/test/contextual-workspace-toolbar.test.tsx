import { useState } from "react"
import { render, screen, waitFor } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { describe, expect, it, vi } from "vitest"

import { ContextualWorkspaceToolbar } from "@/components/layout/AppShell"
import { TooltipProvider } from "@/components/ui/tooltip"

const snapOptions = {
  endpoints: true,
  midpoints: true,
  nearest: false,
  grid: true,
  angles: true,
  axes: true,
  gridSize: 0.1,
  angleIncrement: 15,
}

const hiddenLabels = {
  node: false,
  member: false,
  support: false,
  load: false,
}

function ToolbarHarness({
  activeTool = "select",
  onTool = vi.fn(),
  onToggleSnap = vi.fn(),
  onToggleLabelVisibility = vi.fn(),
  onMemberDrawingOptions = vi.fn(),
  onSnapOptions = vi.fn(),
  onLabelVisibility = vi.fn(),
}: {
  activeTool?: "select" | "node" | "member" | "move" | "split"
  onTool?: (tool: "select" | "node" | "member" | "move" | "split") => void
  onToggleSnap?: () => void
  onToggleLabelVisibility?: () => void
  onMemberDrawingOptions?: (options: { polygonMode: boolean }) => void
  onSnapOptions?: (options: typeof snapOptions) => void
  onLabelVisibility?: (visibility: typeof hiddenLabels) => void
}) {
  const [openMenu, setOpenMenu] = useState<"member-settings" | "snap-settings" | "label-settings" | null>(null)

  return (
    <TooltipProvider>
      <ContextualWorkspaceToolbar
        viewMode="base"
        activePanel={null}
        activeTool={activeTool}
        pendingMemberStart={null}
        editPending={false}
        snapOptions={snapOptions}
        memberDrawingOptions={{ polygonMode: false }}
        labelVisibility={hiddenLabels}
        groupsAvailable={false}
        openToolbarMenu={openMenu}
        onTool={onTool}
        onSnapOptions={onSnapOptions}
        onToggleSnap={onToggleSnap}
        onMemberDrawingOptions={onMemberDrawingOptions}
        onLabelVisibility={onLabelVisibility}
        onToggleLabelVisibility={onToggleLabelVisibility}
        onToolbarMenuOpen={setOpenMenu}
        onTogglePanel={vi.fn()}
      />
    </TooltipProvider>
  )
}

describe("ContextualWorkspaceToolbar", () => {
  it("uses separate icon-only controls for the exclusive editing modes", async () => {
    const user = userEvent.setup()
    const onTool = vi.fn()
    render(<ToolbarHarness onTool={onTool} />)

    const modes = ["Select", "Joint", "Member", "Move", "Split"]
    for (const name of modes) {
      const control = screen.getByRole("button", { name })
      expect(control).toBeInTheDocument()
      expect(control).toHaveTextContent("")
      expect(control.querySelector("svg")).toBeInTheDocument()
    }
    expect(screen.getByRole("group", { name: "Editing mode" })).toHaveAttribute("data-spacing", "2")
    expect(screen.getByRole("button", { name: "Select" })).toHaveAttribute("aria-pressed", "true")
    expect(screen.getByRole("button", { name: "Joint" })).toHaveAttribute("aria-pressed", "false")

    await user.click(screen.getByRole("button", { name: "Joint" }))
    expect(onTool).toHaveBeenCalledWith("node")

    onTool.mockClear()
    await user.click(screen.getByRole("button", { name: "Select" }))
    expect(onTool).not.toHaveBeenCalled()
  })

  it("supports arrow-key focus and keyboard tool selection", async () => {
    const user = userEvent.setup()
    const onTool = vi.fn()
    render(<ToolbarHarness onTool={onTool} />)

    screen.getByRole("button", { name: "Select" }).focus()
    await user.keyboard("{ArrowRight}")
    expect(screen.getByRole("button", { name: "Joint" })).toHaveFocus()
    await user.keyboard(" ")
    expect(onTool).toHaveBeenCalledWith("node")
  })

  it("opens Member settings from an inactive split control without selecting Member", async () => {
    const user = userEvent.setup()
    const onTool = vi.fn()
    const onMemberDrawingOptions = vi.fn()
    render(<ToolbarHarness onTool={onTool} onMemberDrawingOptions={onMemberDrawingOptions} />)

    const member = screen.getByRole("button", { name: "Member" })
    const settings = screen.getByRole("button", { name: "Member settings" })
    expect(screen.getByRole("group", { name: "Member controls" })).toContainElement(member)
    expect(screen.getByRole("group", { name: "Member controls" })).toContainElement(settings)
    expect(member).toHaveAttribute("aria-pressed", "false")
    expect(settings).toHaveAttribute("aria-expanded", "false")

    await user.click(settings)
    expect(settings).toHaveAttribute("aria-expanded", "true")
    expect(await screen.findByText("Configure continuous member drawing.")).toBeVisible()
    expect(member).toHaveAttribute("aria-pressed", "false")
    expect(onTool).not.toHaveBeenCalled()

    await user.click(screen.getByRole("checkbox", { name: "Continuous drawing" }))
    expect(onMemberDrawingOptions).toHaveBeenCalledWith({ polygonMode: true })

    await user.keyboard("{Escape}")
    await waitFor(() => {
      expect(settings).toHaveFocus()
      expect(settings).toHaveAttribute("aria-expanded", "false")
    })
  })

  it("keeps Snap and Label state and settings independent", async () => {
    const user = userEvent.setup()
    const onToggleSnap = vi.fn()
    const onToggleLabelVisibility = vi.fn()
    render(
      <ToolbarHarness
        onToggleSnap={onToggleSnap}
        onToggleLabelVisibility={onToggleLabelVisibility}
      />,
    )

    const snaps = screen.getByRole("button", { name: "Snaps" })
    const labels = screen.getByRole("button", { name: "Labels" })
    expect(snaps).toHaveAttribute("aria-pressed", "true")
    expect(labels).toHaveAttribute("aria-pressed", "false")
    expect(snaps).toHaveTextContent("")
    expect(labels).toHaveTextContent("")

    await user.click(snaps)
    expect(onToggleSnap).toHaveBeenCalledOnce()
    expect(onToggleLabelVisibility).not.toHaveBeenCalled()

    await user.click(labels)
    expect(onToggleLabelVisibility).toHaveBeenCalledOnce()

    const snapSettings = screen.getByRole("button", { name: "Snap settings" })
    const labelSettings = screen.getByRole("button", { name: "Label settings" })
    expect(screen.getByRole("group", { name: "Snap controls" })).toContainElement(snapSettings)
    expect(screen.getByRole("group", { name: "Label controls" })).toContainElement(labelSettings)
    expect(snapSettings).toHaveAttribute("aria-expanded", "false")
    expect(labelSettings).toHaveAttribute("aria-expanded", "false")

    onToggleSnap.mockClear()
    onToggleLabelVisibility.mockClear()
    await user.click(snapSettings)
    expect(snapSettings).toHaveAttribute("aria-expanded", "true")
    expect(await screen.findByText("Snap settings")).toBeVisible()
    expect(screen.getByText("Snapping")).toBeVisible()
    expect(screen.queryByRole("checkbox", { name: "Continuous drawing" })).not.toBeInTheDocument()
    expect(onToggleSnap).not.toHaveBeenCalled()
    expect(onToggleLabelVisibility).not.toHaveBeenCalled()

    await user.keyboard("{Escape}")
    await user.click(labelSettings)
    expect(labelSettings).toHaveAttribute("aria-expanded", "true")
    expect(await screen.findByText("Choose model annotations shown in the viewport.")).toBeVisible()
    expect(screen.getByRole("checkbox", { name: "Node labels" })).toBeVisible()
    expect(screen.queryByText("Snapping")).not.toBeInTheDocument()
    expect(onToggleLabelVisibility).not.toHaveBeenCalled()
  })

  it("keeps horizontal overflow constrained to the toolbar surface", () => {
    render(<ToolbarHarness />)

    const toolbar = screen.getByLabelText("Base model edit tools")
    expect(toolbar).toHaveClass("min-w-0", "flex-nowrap")
    expect(toolbar).not.toHaveClass("overflow-hidden")
  })
})
