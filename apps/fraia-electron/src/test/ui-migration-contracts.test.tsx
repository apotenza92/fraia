import * as React from "react"
import { render, screen, waitFor } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { describe, expect, it, vi } from "vitest"

import { Checkbox } from "@/components/ui/checkbox"
import { Button } from "@/components/ui/button"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog"
import {
  Menubar,
  MenubarContent,
  MenubarGroup,
  MenubarItem,
  MenubarMenu,
  MenubarTrigger,
} from "@/components/ui/menubar"
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip"

describe("shadcn wrapper accessibility contracts", () => {
  it("toggles a labelled checkbox from the keyboard", async () => {
    const user = userEvent.setup()
    const onCheckedChange = vi.fn()

    render(
      <label>
        <Checkbox onCheckedChange={onCheckedChange} />
        Include in comparison
      </label>,
    )

    const checkbox = screen.getByRole("checkbox", {
      name: "Include in comparison",
    })
    checkbox.focus()
    await user.keyboard(" ")

    expect(checkbox).toBeChecked()
    expect(onCheckedChange.mock.calls[0]?.[0]).toBe(true)
  })

  it("moves focus into a dialog and returns it after Escape", async () => {
    const user = userEvent.setup()

    render(
      <Dialog>
        <DialogTrigger>Review option</DialogTrigger>
        <DialogContent>
          <DialogTitle>Option evidence</DialogTitle>
          <DialogDescription>
            Preliminary analysis for the selected revision.
          </DialogDescription>
          <Button type="button">Acknowledge</Button>
        </DialogContent>
      </Dialog>,
    )

    const trigger = screen.getByRole("button", { name: "Review option" })
    await user.click(trigger)

    const dialog = await screen.findByRole("dialog", {
      name: "Option evidence",
    })
    expect(dialog).toBeVisible()
    await waitFor(() =>
      expect(dialog).toContainElement(document.activeElement as HTMLElement),
    )

    await user.keyboard("{Escape}")
    await waitFor(() => expect(dialog).not.toBeInTheDocument())
    expect(trigger).toHaveFocus()
  })

  it("keeps tab selection and panel visibility in sync", async () => {
    const user = userEvent.setup()

    render(
      <Tabs defaultValue="assumptions">
        <TabsList aria-label="Option inspector" activateOnFocus>
          <TabsTrigger value="assumptions">Assumptions</TabsTrigger>
          <TabsTrigger value="evidence">Evidence</TabsTrigger>
        </TabsList>
        <TabsContent value="assumptions">Assumption summary</TabsContent>
        <TabsContent value="evidence">Analysis evidence</TabsContent>
      </Tabs>,
    )

    const evidenceTab = screen.getByRole("tab", { name: "Evidence" })
    await user.click(evidenceTab)

    expect(evidenceTab).toHaveAttribute("aria-selected", "true")
    expect(document.getElementById(evidenceTab.getAttribute("aria-controls") ?? "")).toHaveTextContent("Analysis evidence")

    await user.keyboard("{ArrowLeft}")
    const assumptionsTab = screen.getByRole("tab", { name: "Assumptions" })
    expect(assumptionsTab).toHaveFocus()
    expect(assumptionsTab).toHaveAttribute("aria-selected", "true")
    expect(document.getElementById(assumptionsTab.getAttribute("aria-controls") ?? "")).toHaveTextContent("Assumption summary")

    await user.keyboard("{End}")
    expect(evidenceTab).toHaveFocus()
    expect(evidenceTab).toHaveAttribute("aria-selected", "true")
    await user.keyboard("{Home}")
    expect(assumptionsTab).toHaveFocus()
    expect(assumptionsTab).toHaveAttribute("aria-selected", "true")
  })

  it("shows Select item labels and reports keyboard selection", async () => {
    const user = userEvent.setup()
    const onValueChange = vi.fn()

    render(
      <Select
        defaultValue="gpt-5.5"
        items={[
          { value: "gpt-5.5", label: "GPT-5.5" },
          { value: "gpt-5.6", label: "GPT-5.6" },
        ]}
        onValueChange={onValueChange}
      >
        <SelectTrigger aria-label="Model">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectGroup>
            <SelectItem value="gpt-5.5">GPT-5.5</SelectItem>
            <SelectItem value="gpt-5.6">GPT-5.6</SelectItem>
          </SelectGroup>
        </SelectContent>
      </Select>,
    )

    const trigger = screen.getByRole("combobox", { name: "Model" })
    expect(trigger).toHaveTextContent("GPT-5.5")
    expect(trigger).not.toHaveTextContent("gpt-5.5")

    trigger.focus()
    await user.keyboard("{ArrowDown}")
    await user.keyboard("gpt-5.6")
    await user.keyboard("{Enter}")

    expect(onValueChange).toHaveBeenLastCalledWith("gpt-5.6", expect.anything())
    expect(trigger).toHaveTextContent("GPT-5.6")
    await waitFor(() => expect(trigger).toHaveFocus())
  })

  it("opens contextual help on focus and dismisses it with Escape", async () => {
    const user = userEvent.setup()

    render(
      <TooltipProvider>
        <Tooltip>
          <TooltipTrigger>Analysis status</TooltipTrigger>
          <TooltipContent>Current preliminary analysis</TooltipContent>
        </Tooltip>
      </TooltipProvider>,
    )

    const trigger = screen.getByRole("button", { name: "Analysis status" })
    await user.tab()
    expect(trigger).toHaveFocus()
    expect(
      await screen.findByText("Current preliminary analysis"),
    ).toBeVisible()

    await user.keyboard("{Escape}")
    await waitFor(() =>
      expect(
        screen.queryByText("Current preliminary analysis"),
      ).not.toBeInTheDocument(),
    )
  })

  it("composes tooltip and popover behavior onto one named button", async () => {
    const user = userEvent.setup()

    render(
      <TooltipProvider>
        <Popover>
          <Tooltip>
            <TooltipTrigger
              render={(
                <PopoverTrigger
                  render={<Button aria-label="Snap settings">Snaps</Button>}
                />
              )}
            />
            <TooltipContent>Configure snap behavior</TooltipContent>
          </Tooltip>
          <PopoverContent>
            <Button type="button">Toggle intersections</Button>
          </PopoverContent>
        </Popover>
      </TooltipProvider>,
    )

    const trigger = screen.getByRole("button", { name: "Snap settings" })
    expect(
      screen.getAllByRole("button", { name: "Snap settings" }),
    ).toHaveLength(1)

    await user.hover(trigger)
    expect(await screen.findByText("Configure snap behavior")).toBeVisible()

    await user.click(trigger)
    expect(
      await screen.findByRole("button", { name: "Toggle intersections" }),
    ).toBeVisible()
  })

  it("supports keyboard entry and dismissal for popovers and menus", async () => {
    const user = userEvent.setup()
    const onOpenProject = vi.fn()

    render(
      <>
        <Popover>
          <PopoverTrigger>Option filters</PopoverTrigger>
          <PopoverContent>
            <Button type="button">Reset filters</Button>
          </PopoverContent>
        </Popover>
        <Menubar aria-label="Application menu">
          <MenubarMenu>
            <MenubarTrigger>File</MenubarTrigger>
            <MenubarContent>
              <MenubarGroup>
                <MenubarItem onClick={onOpenProject}>Open project</MenubarItem>
              </MenubarGroup>
            </MenubarContent>
          </MenubarMenu>
        </Menubar>
      </>,
    )

    const popoverTrigger = screen.getByRole("button", {
      name: "Option filters",
    })
    popoverTrigger.focus()
    await user.keyboard("{Enter}")
    expect(await screen.findByRole("button", { name: "Reset filters" })).toBeVisible()
    await user.keyboard("{Escape}")
    await waitFor(() => expect(popoverTrigger).toHaveFocus())

    const menuTrigger = screen.getByRole("menuitem", { name: "File" })
    menuTrigger.focus()
    await user.keyboard("{ArrowDown}")
    const menuItem = await screen.findByRole("menuitem", {
      name: "Open project",
    })
    await waitFor(() => expect(menuItem).toHaveFocus())
    await user.keyboard("{Enter}")
    expect(onOpenProject).toHaveBeenCalledOnce()
    await waitFor(() => expect(menuItem).not.toBeInTheDocument())
    expect(menuTrigger).toHaveFocus()
  })
})
