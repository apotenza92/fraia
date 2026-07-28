import { defineConfig } from "@playwright/test"

export default defineConfig({
  testDir: "./tests/electron",
  outputDir: "/tmp/fraia-playwright-results",
  fullyParallel: false,
  workers: 1,
  retries: 0,
  timeout: 60_000,
  expect: {
    timeout: 10_000,
  },
  reporter: "line",
})
