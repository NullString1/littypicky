import { test, expect } from "@playwright/test";

test.describe("API Connection Test", () => {
  test("should connect to backend test endpoint from Node.js", async () => {
    // This tests the Node.js -> Backend connection (test helpers)
    const response = await fetch("http://localhost:8080/api/test/status");
    expect(response.ok).toBe(true);

    const data = await response.json();
    expect(data.success).toBe(true);
    expect(data.message).toContain("Test helpers are enabled");
  });

  test("should connect to backend through Vite proxy from browser", async ({
    page,
  }) => {
    // This tests the Browser -> Vite -> Backend connection (app requests)
    await page.goto("/");

    // Make API request from browser context
    const response = await page.evaluate(async () => {
      const res = await fetch("/api/test/status");
      return {
        ok: res.ok,
        status: res.status,
        data: await res.json(),
      };
    });

    console.log("Browser API response:", response);
    expect(response.ok).toBe(true);
    expect(response.data.success).toBe(true);
  });

  test("should access backend health endpoint", async ({ page }) => {
    await page.goto("/");

    const response = await page.evaluate(async () => {
      const res = await fetch("/api/health");
      return {
        ok: res.ok,
        status: res.status,
        statusText: res.statusText,
      };
    });

    console.log("Health check response:", response);
    expect(response.ok).toBe(true);
  });

  test("should log network requests during registration", async ({ page }) => {
    // Enable request logging
    page.on("request", (request) => {
      if (request.url().includes("/api/")) {
        console.log(">>>", request.method(), request.url());
      }
    });

    page.on("response", (response) => {
      if (response.url().includes("/api/")) {
        console.log("<<<", response.status(), response.url());
      }
    });

    page.on("requestfailed", (request) => {
      console.log("XXX FAILED", request.url(), request.failure()?.errorText);
    });

    // Try to load registration page
    await page.goto("/auth/register");

    // Check if page loaded
    await expect(
      page.locator('h2:has-text("Join the movement")'),
    ).toBeVisible();

    console.log("Registration page loaded successfully");
  });
});
