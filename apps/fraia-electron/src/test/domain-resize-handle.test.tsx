import { render, screen } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { describe, expect, it, vi } from "vitest"

import { ResizeHandle } from "@/components/domain-ui/ResizeHandle"

describe("ResizeHandle domain exception", () => {
  it("exposes separator values and supports Arrow, Home, and End resizing", async () => {
    const user = userEvent.setup()
    const onValueChange = vi.fn()

    render(
      <ResizeHandle
        label="Resize workspace split"
        min={300}
        max={640}
        value={430}
        handleStyle={{ left: 430, width: 8 }}
        separatorStyle={{ left: 430 }}
        onPointerDown={vi.fn()}
        onValueChange={onValueChange}
      />,
    )

    const handle = screen.getByRole("separator", { name: "Resize workspace split" })
    expect(handle).toHaveAttribute("data-domain-ui", "resize-handle")
    expect(handle).toHaveAttribute("aria-valuemin", "300")
    expect(handle).toHaveAttribute("aria-valuemax", "640")
    expect(handle).toHaveAttribute("aria-valuenow", "430")

    handle.focus()
    await user.keyboard("{ArrowLeft}{ArrowRight}{Home}{End}")
    expect(onValueChange.mock.calls.map(([value]) => value)).toEqual([414, 446, 300, 640])
  })
})
