import type { CSSProperties, KeyboardEvent, PointerEvent } from "react"

import { Separator } from "@/components/ui/separator"

/**
 * Domain UI exception: Fraia's renderer and workspace panels are independently
 * positioned around a WebGL viewport and preserve a ratio across window sizes.
 * The official ResizablePanelGroup must own the panel layout, so it cannot expose
 * this existing overlay geometry without changing renderer behavior. The visible
 * rule still comes from the official Separator; this component owns interaction
 * geometry and accessible value changes only.
 */
export function ResizeHandle({
  label,
  min,
  max,
  value,
  handleStyle,
  separatorStyle,
  onPointerDown,
  onValueChange,
}: {
  label: string
  min: number
  max: number
  value: number
  handleStyle: CSSProperties
  separatorStyle?: CSSProperties
  onPointerDown: (event: PointerEvent<HTMLDivElement>) => void
  onValueChange: (value: number) => void
}) {
  function handleKeyDown(event: KeyboardEvent<HTMLDivElement>) {
    let nextValue: number | null = null
    if (event.key === "ArrowLeft") nextValue = value - 16
    if (event.key === "ArrowRight") nextValue = value + 16
    if (event.key === "Home") nextValue = min
    if (event.key === "End") nextValue = max
    if (nextValue == null) return
    event.preventDefault()
    onValueChange(Math.min(max, Math.max(min, nextValue)))
  }

  return (
    <>
      {separatorStyle ? (
        <Separator
          aria-hidden="true"
          orientation="vertical"
          className="pointer-events-none absolute top-0 h-full"
          style={separatorStyle}
        />
      ) : null}
      <div
        data-domain-ui="resize-handle"
        role="separator"
        aria-orientation="vertical"
        aria-label={label}
        aria-valuemin={Math.round(min)}
        aria-valuemax={Math.round(max)}
        aria-valuenow={Math.round(value)}
        tabIndex={0}
        className="absolute top-0 h-full cursor-col-resize"
        style={handleStyle}
        onKeyDown={handleKeyDown}
        onPointerDown={onPointerDown}
      />
    </>
  )
}
