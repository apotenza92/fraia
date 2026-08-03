import type { ComponentProps } from "react"

import { Button } from "@/components/ui/button"
import { cn } from "@/lib/utils"

const RESTING_SURFACE = "bg-transparent! hover:bg-muted! aria-expanded:bg-transparent! dark:bg-transparent! dark:hover:bg-muted/50! dark:aria-expanded:bg-transparent!"
const SELECTED_SURFACE = "bg-muted! hover:bg-muted! aria-expanded:bg-muted! dark:bg-muted! dark:hover:bg-muted! dark:aria-expanded:bg-muted!"

function splitButtonSegmentSurface(selected: boolean) {
  return selected ? SELECTED_SURFACE : RESTING_SURFACE
}

/**
 * Domain UI exception: the official shadcn ButtonGroup does not expose a way
 * to mirror an adjacent Toggle's selected surface onto a non-toggle settings
 * segment. This wrapper keeps Button semantics and official geometry while
 * synchronising only that surface without adding a false pressed state.
 */
export function SplitButtonSegment({
  selected = false,
  className,
  ...props
}: ComponentProps<typeof Button> & { selected?: boolean }) {
  return (
    <Button
      data-domain-ui-exception="split-button-segment"
      data-selected={selected || undefined}
      className={cn(splitButtonSegmentSurface(selected), className)}
      {...props}
    />
  )
}
