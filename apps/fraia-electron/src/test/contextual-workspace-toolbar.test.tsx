import { useState } from "react"
import { render, screen } from "@testing-library/react"
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
}: {
  activeTool?: "select" | "node" | "member" | "move" | "split"
  onTool?: (tool: "select" | "node" | "member" | "move" | "split") => void
  onToggleSnap?: () => void
  onToggleLabelVisibility?: () => void
}) {
  const [openMenu, setOpenMenu] = useState<"viewport-settings" | null>(null)

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
        onSnapOptions={vi.fn()}
        onToggleSnap={onToggleSnap}
        onMemberDrawingOptions={vi.fn()}
        onLabelVisibility={vi.fn()}
        onToggleLabelVisibility={onToggleLabelVisibility}
        onToolbarMenuOpen={setOpenMenu}
        onTogglePanel={vi.fn()}
      />
    </TooltipProvider>
  )
}

describe("ContextualWorkspaceToolbar", () => {
  it("uses one exclusive editing group with professional UI labels", async () => {
    const user = userEvent.setup()
    const onTool = vi.fn()
    render(<ToolbarHarness onTool={onTool} />)

    const modes = ["Select", "Joint", "Member", "Move", "Split"]
    for (const name of modes) {
      expect(screen.getByRole("button", { name })).toBeInTheDocument()
    }
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

  it("keeps Snaps and Labels independent and provides one settings surface", async () => {
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

    await user.click(snaps)
    expect(onToggleSnap).toHaveBeenCalledOnce()
    expect(onToggleLabelVisibility).not.toHaveBeenCalled()

    await user.click(labels)
    expect(onToggleLabelVisibility).toHaveBeenCalledOnce()

    expect(screen.queryByRole("button", { name: "Snap settings" })).not.toBeInTheDocument()
    expect(screen.queryByRole("button", { name: "Label settings" })).not.toBeInTheDocument()
    expect(screen.getAllByRole("button", { name: "Toolbar settings" })).toHaveLength(1)

    await user.click(screen.getByRole("button", { name: "Toolbar settings" }))
    expect(await screen.findByText("Viewport settings")).toBeVisible()
    expect(screen.getByText("Member drawing")).toBeVisible()
    expect(screen.getByText("Snapping")).toBeVisible()
    expect(screen.getAllByText("Labels").some((element) => element.className.includes("font-medium"))).toBe(true)
    expect(screen.getByRole("checkbox", { name: "Continuous drawing" })).toBeVisible()
  })
})
