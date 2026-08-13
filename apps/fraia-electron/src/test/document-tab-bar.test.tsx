import { useState } from "react"
import { fireEvent, render, screen, waitFor } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { describe, expect, it } from "vitest"

import {
  DocumentTabBar,
  documentTabTriggerId,
  type DocumentTab,
} from "@/components/domain-ui/DocumentTabBar"
import { CHROME } from "@/components/layout/chromeMetrics"

const initialTabs: DocumentTab[] = [
  { id: "current", label: "Current Model", closable: false, reorderable: false },
  { id: "option-a", label: "Option A", closable: true, reorderable: true },
  { id: "option-b", label: "Option B", closable: true, reorderable: true },
]
const additionalTab: DocumentTab = {
  id: "option-c",
  label: "Option C",
  closable: true,
  reorderable: true,
}

function DocumentTabsHarness({ initialValue = "option-a" }: { initialValue?: string }) {
  const [tabs, setTabs] = useState(initialTabs)
  const [value, setValue] = useState(initialValue)
  const [blankModels, setBlankModels] = useState(0)

  return (
    <>
      <DocumentTabBar
        tabs={tabs}
        value={value}
        panelId="workspace-panel"
        onValueChange={setValue}
        onClose={(id) => setTabs((current) => current.filter((tab) => tab.id !== id))}
        onReorder={(orderedIds) => {
          setTabs((current) => orderedIds.flatMap((id) => {
            const tab = current.find((candidate) => candidate.id === id)
            return tab ? [tab] : []
          }))
        }}
        onOpen={() => setTabs((current) => current.some((tab) => tab.id === additionalTab.id)
          ? current
          : [...current, additionalTab])}
        onNewBlankModel={() => setBlankModels((current) => current + 1)}
      />
      <span data-testid="blank-models-created">{blankModels}</span>
      <div
        id="workspace-panel"
        role="tabpanel"
        aria-labelledby={documentTabTriggerId(value)}
      />
    </>
  )
}

function dataTransferStub() {
  const values = new Map<string, string>()
  return {
    effectAllowed: "none",
    setData(type: string, value: string) {
      values.set(type, value)
    },
    getData(type: string) {
      return values.get(type) ?? ""
    },
  }
}

describe("DocumentTabBar", () => {
  it("uses the reviewed full-width Nova tab bar and keeps the active panel relationship current", () => {
    render(<DocumentTabsHarness />)

    const tabList = screen.getByRole("tablist", { name: "Open documents" })
    const activeTab = screen.getByRole("tab", { name: "Option A" })
    const inactiveTab = screen.getByRole("tab", { name: "Option B" })
    const panel = screen.getByRole("tabpanel")
    const bar = tabList.closest('[data-domain-ui="document-tabs"]')

    expect(bar).toHaveClass("h-full", "border-b", "bg-background", "p-2")
    expect(CHROME.tabHeight).toBe(49)
    expect(tabList).toHaveAttribute("data-variant", "default")
    expect(tabList).toHaveClass("bg-background!", "rounded-none", "p-0!", "gap-2")
    expect(activeTab).toHaveClass(
      "h-8!",
      "bg-background!",
      "data-active:bg-muted!",
    )
    expect(activeTab.parentElement).toHaveAttribute("data-slot", "tabs-list")
    expect(tabList.closest('[data-document-tab-scroll]')).toHaveClass("flex", "h-8", "items-center", "gap-2", "overflow-x-auto", "no-scrollbar")
    expect(activeTab).toHaveAttribute("aria-selected", "true")
    expect(inactiveTab).toHaveAttribute("aria-selected", "false")
    expect(activeTab).not.toHaveClass("max-w-70", "pr-7")
    expect(activeTab).toHaveClass("pr-8")
    expect(inactiveTab).not.toHaveClass("max-w-70", "pr-7")
    expect(activeTab).toHaveAttribute("aria-controls", panel.id)
    expect(panel).toHaveAttribute("aria-labelledby", activeTab.id)
  })

  it("pins the open and blank-model actions immediately after the newest tab", async () => {
    const user = userEvent.setup()
    render(<DocumentTabsHarness />)

    const tabList = screen.getByRole("tablist", { name: "Open documents" })
    const openTab = screen.getByRole("button", { name: "Open Fraia model" })
    const newBlankModel = screen.getByRole("button", { name: "New blank model" })
    expect(openTab).toHaveAttribute("data-slot", "tooltip-trigger")
    expect(openTab).toHaveAttribute("data-document-tab-open")
    expect(openTab.querySelector("svg")).toHaveClass("lucide-plus")
    expect(openTab).toHaveTextContent("")
    expect(openTab).toHaveClass(
      "size-8",
      "border-border",
      "bg-background!",
      "hover:bg-muted!",
    )
    expect(newBlankModel).toHaveAttribute("data-slot", "tooltip-trigger")
    expect(newBlankModel.querySelector("svg")).toHaveClass("lucide-file-plus-corner")
    expect(newBlankModel).toHaveTextContent("")
    expect(newBlankModel).toHaveClass("size-8", "border-border", "bg-background!", "hover:bg-muted!")
    expect(tabList).toHaveClass("group-data-horizontal/tabs:h-8")
    expect(tabList.parentElement?.nextElementSibling).toBe(openTab)
    expect(openTab.nextElementSibling).toBe(newBlankModel)

    await user.click(openTab)

    expect(screen.getAllByRole("tab").map((tab) => tab.textContent)).toEqual([
      "Current Model",
      "Option A",
      "Option B",
      "Option C",
    ])
    expect(tabList.parentElement?.nextElementSibling).toBe(openTab)
    expect(openTab.nextElementSibling).toBe(newBlankModel)
    expect(openTab).toBeEnabled()

    await user.click(newBlankModel)
    expect(screen.getByTestId("blank-models-created")).toHaveTextContent("1")
  })

  it("keeps official close buttons separate from tab triggers", async () => {
    const user = userEvent.setup()
    render(<DocumentTabsHarness />)

    const activeClose = screen.getByRole("button", { name: "Close Option A" })
    const inactiveClose = screen.getByRole("button", { name: "Close Option B" })
    const currentTab = screen.getByRole("tab", { name: "Current Model" })
    const inactiveTab = screen.getByRole("tab", { name: "Option B" })

    expect(screen.queryByRole("button", { name: "Close Current Model" })).not.toBeInTheDocument()
    expect(screen.getByRole("tab", { name: "Option A" })).not.toContainElement(activeClose)
    expect(inactiveTab).not.toContainElement(inactiveClose)
    expect(activeClose).toBeVisible()
    expect(inactiveClose).toBeVisible()

    currentTab.focus()
    await user.keyboard("{Control>}{Shift>}{ArrowRight}{/Shift}{/Control}")
    expect(screen.getAllByRole("tab").map((tab) => tab.textContent)).toEqual([
      "Current Model",
      "Option A",
      "Option B",
    ])
  })

  it("selects a neighbour and recovers focus after closing the active tab", async () => {
    const user = userEvent.setup()
    render(<DocumentTabsHarness />)

    await user.click(screen.getByRole("button", { name: "Close Option A" }))

    expect(screen.queryByRole("tab", { name: "Option A" })).not.toBeInTheDocument()
    const nextTab = screen.getByRole("tab", { name: "Option B" })
    expect(nextTab).toHaveAttribute("aria-selected", "true")
    await waitFor(() => expect(nextTab).toHaveFocus())
    expect(screen.getByRole("status", { hidden: true })).toHaveTextContent("Closed Option A")
  })

  it("reorders option tabs with native drag and drop without moving the pinned model", () => {
    render(<DocumentTabsHarness />)
    const optionA = screen.getByRole("tab", { name: "Option A" })
    const optionB = screen.getByRole("tab", { name: "Option B" })
    const source = optionA as HTMLElement
    const target = optionB as HTMLElement
    const dataTransfer = dataTransferStub()

    fireEvent.dragStart(source, { dataTransfer })
    fireEvent.dragOver(target, { dataTransfer })
    fireEvent.drop(target, { dataTransfer })

    expect(screen.getAllByRole("tab").map((tab) => tab.textContent)).toEqual([
      "Current Model",
      "Option B",
      "Option A",
    ])
  })

  it("supports keyboard reordering while preserving ordinary roving selection", async () => {
    const user = userEvent.setup()
    render(<DocumentTabsHarness />)

    const optionA = screen.getByRole("tab", { name: "Option A" })
    optionA.focus()
    await user.keyboard("{Control>}{Shift>}{ArrowRight}{/Shift}{/Control}")

    expect(screen.getAllByRole("tab").map((tab) => tab.textContent)).toEqual([
      "Current Model",
      "Option B",
      "Option A",
    ])
    await waitFor(() => expect(optionA).toHaveFocus())

    await user.keyboard("{ArrowLeft}")
    const optionB = screen.getByRole("tab", { name: "Option B" })
    await waitFor(() => expect(optionB).toHaveFocus())
    expect(optionB).toHaveAttribute("aria-selected", "true")

    await user.keyboard("{Home}")
    const current = screen.getByRole("tab", { name: "Current Model" })
    await waitFor(() => expect(current).toHaveFocus())
    expect(current).toHaveAttribute("aria-selected", "true")

    await user.keyboard("{End}")
    await waitFor(() => expect(optionA).toHaveFocus())
    expect(optionA).toHaveAttribute("aria-selected", "true")
  })
})
