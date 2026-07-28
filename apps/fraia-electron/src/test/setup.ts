import "@testing-library/jest-dom/vitest"
import { cleanup } from "@testing-library/react"
import { afterEach } from "vitest"

afterEach(() => cleanup())

class ResizeObserverMock implements ResizeObserver {
  observe() {}
  unobserve() {}
  disconnect() {}
}

Object.defineProperty(window, "ResizeObserver", {
  configurable: true,
  writable: true,
  value: ResizeObserverMock,
})

Object.defineProperty(window, "matchMedia", {
  configurable: true,
  writable: true,
  value: (query: string): MediaQueryList => ({
    matches: false,
    media: query,
    onchange: null,
    addEventListener() {},
    removeEventListener() {},
    addListener() {},
    removeListener() {},
    dispatchEvent: () => false,
  }),
})

if (!window.PointerEvent) {
  Object.defineProperty(window, "PointerEvent", {
    configurable: true,
    writable: true,
    value: MouseEvent,
  })
}

Object.defineProperty(Element.prototype, "scrollIntoView", {
  configurable: true,
  writable: true,
  value() {},
})

Object.defineProperty(Element.prototype, "getAnimations", {
  configurable: true,
  writable: true,
  value: () => [],
})

for (const method of [
  "hasPointerCapture",
  "setPointerCapture",
  "releasePointerCapture",
] as const) {
  if (!(method in Element.prototype)) {
    Object.defineProperty(Element.prototype, method, {
      configurable: true,
      writable: true,
      value: method === "hasPointerCapture" ? () => false : () => {},
    })
  }
}
