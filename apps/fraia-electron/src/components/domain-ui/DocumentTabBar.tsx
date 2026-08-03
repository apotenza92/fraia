import { useId, useRef, useState } from "react"
import type { DragEvent, KeyboardEvent, MouseEvent } from "react"
import { FilePlus2, Plus, X } from "lucide-react"

import { Button } from "@/components/ui/button"
import { Separator } from "@/components/ui/separator"
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip"
import { cn } from "@/lib/utils"

export type DocumentTab = {
  id: string
  label: string
  closable?: boolean
  reorderable?: boolean
}

export function documentTabTriggerId(id: string) {
  return `fraia-document-tab-${encodeURIComponent(id)}`
}

function reorderedTabIds(tabs: DocumentTab[], sourceId: string, targetId: string) {
  const sourceIndex = tabs.findIndex((tab) => tab.id === sourceId)
  const targetIndex = tabs.findIndex((tab) => tab.id === targetId)
  if (sourceIndex < 0 || targetIndex < 0 || sourceIndex === targetIndex) return null
  if (tabs[sourceIndex].reorderable === false || tabs[targetIndex].reorderable === false) return null

  const next = tabs.map((tab) => tab.id)
  const [moved] = next.splice(sourceIndex, 1)
  next.splice(targetIndex, 0, moved)
  return next
}

/**
 * Domain UI exception: the official shadcn/Base UI Tabs component does not
 * provide document closing, reordering, opening, creation, or close-focus recovery. Fraia
 * keeps selection and roving focus in official Tabs, renders close and document
 * actions as separate official Buttons, and owns only the document interaction.
 */
export function DocumentTabBar({
  tabs,
  value,
  panelId,
  onValueChange,
  onClose,
  onReorder,
  onOpen,
  openDisabled = false,
  onNewBlankModel,
  newBlankModelDisabled = false,
}: {
  tabs: DocumentTab[]
  value: string
  panelId: string
  onValueChange: (value: string) => void
  onClose: (value: string) => void
  onReorder: (orderedIds: string[]) => void
  onOpen: () => void
  openDisabled?: boolean
  onNewBlankModel: () => void
  newBlankModelDisabled?: boolean
}) {
  const instructionsId = useId()
  const draggedIdRef = useRef<string | null>(null)
  const [announcement, setAnnouncement] = useState("")

  function focusTab(id: string) {
    const schedule = window.requestAnimationFrame ?? ((callback: FrameRequestCallback) => window.setTimeout(callback, 0))
    schedule(() => document.getElementById(documentTabTriggerId(id))?.focus())
  }

  function announceMove(id: string, orderedIds: string[]) {
    const tab = tabs.find((candidate) => candidate.id === id)
    const position = orderedIds.indexOf(id) + 1
    setAnnouncement(`Moved ${tab?.label ?? "tab"} to position ${position} of ${orderedIds.length}.`)
  }

  function moveTab(sourceId: string, targetId: string, restoreFocus = true) {
    const next = reorderedTabIds(tabs, sourceId, targetId)
    if (!next) return
    onReorder(next)
    announceMove(sourceId, next)
    if (restoreFocus) focusTab(sourceId)
  }

  function handleDragStart(event: DragEvent<HTMLDivElement>, tab: DocumentTab) {
    const startedFromClose = event.target instanceof Element && event.target.closest("[data-document-tab-close]")
    if (tab.reorderable === false || startedFromClose) {
      event.preventDefault()
      return
    }
    draggedIdRef.current = tab.id
    event.dataTransfer.effectAllowed = "move"
    event.dataTransfer.setData("text/plain", tab.id)
  }

  function handleDrop(event: DragEvent<HTMLDivElement>, targetId: string) {
    event.preventDefault()
    const sourceId = event.dataTransfer.getData("text/plain") || draggedIdRef.current
    draggedIdRef.current = null
    if (sourceId) moveTab(sourceId, targetId)
  }

  function handleTabKeyDown(event: KeyboardEvent, tab: DocumentTab) {
    const isReorderKey = (event.metaKey || event.ctrlKey)
      && event.shiftKey
      && (event.key === "ArrowLeft" || event.key === "ArrowRight")
    if (isReorderKey) {
      event.preventDefault()
      if (tab.reorderable === false) return
      const currentIndex = tabs.findIndex((candidate) => candidate.id === tab.id)
      const targetIndex = event.key === "ArrowLeft" ? currentIndex - 1 : currentIndex + 1
      const target = tabs[targetIndex]
      if (!target || target.reorderable === false) return
      moveTab(tab.id, target.id, false)
      return
    }

    if (event.altKey || event.ctrlKey || event.metaKey || event.shiftKey) return
    const currentIndex = tabs.findIndex((candidate) => candidate.id === tab.id)
    let target: DocumentTab | undefined
    if (event.key === "Home") target = tabs[0]
    if (event.key === "End") target = tabs[tabs.length - 1]
    if (event.key === "ArrowLeft") target = tabs[(currentIndex - 1 + tabs.length) % tabs.length]
    if (event.key === "ArrowRight") target = tabs[(currentIndex + 1) % tabs.length]
    if (!target) return

    event.preventDefault()
    onValueChange(target.id)
    focusTab(target.id)
  }

  function handleClose(event: MouseEvent, tab: DocumentTab) {
    event.preventDefault()
    event.stopPropagation()
    const closingIndex = tabs.findIndex((candidate) => candidate.id === tab.id)
    const nextTab = tabs[closingIndex + 1] ?? tabs[closingIndex - 1]
    const focusId = tab.id === value ? nextTab?.id : value

    if (tab.id === value && nextTab) onValueChange(nextTab.id)
    onClose(tab.id)
    setAnnouncement(`Closed ${tab.label}.`)
    if (focusId) focusTab(focusId)
  }

  return (
    <div
      data-domain-ui="document-tabs"
      className="flex h-full min-w-0 shrink-0 items-center border-b border-border bg-background p-2"
    >
      <p id={instructionsId} className="sr-only">
        Use Arrow keys, Home, and End to navigate tabs. Hold Control or Command and Shift with an Arrow key to reorder the focused tab.
      </p>
      <Tabs value={value} onValueChange={onValueChange} className="h-8 min-w-0 flex-1 gap-0">
        <div
          data-document-tab-scroll
          className="no-scrollbar flex h-8 min-w-0 items-center gap-2 overflow-x-auto"
        >
          <TabsList
            aria-label="Open documents"
            activateOnFocus
            className="shrink-0 justify-start gap-2 rounded-none bg-background! p-0! group-data-horizontal/tabs:h-8!"
          >
            {tabs.map((tab) => (
              <div
                  key={tab.id}
                  role="presentation"
                  draggable={tab.reorderable !== false}
                  className="group/document-tab relative flex h-8 shrink-0 items-center"
                  onDragStart={(event) => handleDragStart(event, tab)}
                  onDragEnd={() => {
                    draggedIdRef.current = null
                  }}
                  onDragOver={(event) => {
                    if (tab.reorderable !== false) event.preventDefault()
                  }}
                  onDrop={(event) => handleDrop(event, tab.id)}
                >
                  <TabsTrigger
                    id={documentTabTriggerId(tab.id)}
                    aria-controls={panelId}
                    aria-describedby={instructionsId}
                    aria-keyshortcuts="Control+Shift+ArrowLeft Control+Shift+ArrowRight Meta+Shift+ArrowLeft Meta+Shift+ArrowRight"
                    value={tab.id}
                    title={tab.label}
                    className={cn(
                      "h-8! bg-background! data-active:bg-muted!",
                      tab.closable && "pr-8",
                    )}
                    onKeyDown={(event) => handleTabKeyDown(event, tab)}
                  >
                    {tab.label}
                  </TabsTrigger>
                  {tab.closable ? (
                    <Button
                      data-document-tab-close
                      type="button"
                      variant="ghost"
                      size="icon-xs"
                      aria-label={`Close ${tab.label}`}
                      title={`Close ${tab.label}`}
                      className="absolute right-1 top-1/2 -translate-y-1/2"
                      onPointerDown={(event) => event.stopPropagation()}
                      onClick={(event) => handleClose(event, tab)}
                    >
                      <X />
                    </Button>
                  ) : null}
              </div>
            ))}
          </TabsList>
          {tabs.length > 0 ? (
            <Separator
              orientation="vertical"
              data-document-tab-actions-separator
            />
          ) : null}
          <Tooltip>
            <TooltipTrigger
              render={(
                <Button
                  data-document-tab-open
                  type="button"
                  variant="outline"
                  size="icon"
                  aria-label="Open Fraia model"
                  disabled={openDisabled}
                  className="shrink-0 bg-background! hover:bg-muted!"
                  onClick={onOpen}
                >
                  <Plus />
                </Button>
              )}
            />
            <TooltipContent>Open Fraia model</TooltipContent>
          </Tooltip>
          <Tooltip>
            <TooltipTrigger
              render={(
                <Button
                  data-document-tab-new-blank
                  type="button"
                  variant="outline"
                  size="icon"
                  aria-label="New blank model"
                  disabled={newBlankModelDisabled}
                  className="shrink-0 bg-background! hover:bg-muted!"
                  onClick={onNewBlankModel}
                >
                  <FilePlus2 />
                </Button>
              )}
            />
            <TooltipContent>New blank model</TooltipContent>
          </Tooltip>
        </div>
      </Tabs>
      <p role="status" className="sr-only" aria-live="polite" aria-atomic="true">
        {announcement}
      </p>
    </div>
  )
}
