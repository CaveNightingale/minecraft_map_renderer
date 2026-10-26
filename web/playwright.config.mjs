import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./test",
  testMatch: "*.spec.mjs",
  fullyParallel: false,
  workers: 1,
  use: {
    baseURL: "http://127.0.0.1:4174",
    viewport: { width: 800, height: 600 },
    deviceScaleFactor: 1,
  },
  webServer: {
    command: "node test/server.mjs",
    url: "http://127.0.0.1:4174",
    reuseExistingServer: false,
  },
});
