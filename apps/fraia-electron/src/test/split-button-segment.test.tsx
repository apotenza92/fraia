import { render, screen } from "@testing-library/react"
import { describe, expect, it } from "vitest"

import { SplitButtonSegment } from "@/components/domain-ui/SplitButtonSegment"

describe("SplitButtonSegment", () => {
  it("mirrors selection without claiming that the settings action is pressed", () => {
    const { rerender } = render(
      <SplitButtonSegment aria-label="Member settings" aria-expanded={false} selected variant="outline" size="icon" />,
    )

    const settings = screen.getByRole("button", { name: "Member settings" })
    expect(settings).toHaveAttribute("data-domain-ui-exception", "split-button-segment")
    expect(settings).toHaveAttribute("data-selected", "true")
    expect(settings).toHaveClass("bg-muted!")
    expect(settings).not.toHaveAttribute("aria-pressed")
    expect(settings).toHaveAttribute("aria-expanded", "false")

    rerender(
      <SplitButtonSegment aria-label="Member settings" aria-expanded selected={false} variant="outline" size="icon" />,
    )
    expect(settings).not.toHaveAttribute("data-selected")
    expect(settings).toHaveClass("bg-transparent!")
    expect(settings).not.toHaveAttribute("aria-pressed")
    expect(settings).toHaveAttribute("aria-expanded", "true")
  })
})
