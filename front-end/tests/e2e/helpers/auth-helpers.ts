import { Page, expect } from "@playwright/test";

/**
 * Helper functions for authentication flows in e2e tests
 */

export interface UserCredentials {
  email: string;
  password: string;
  full_name: string;
  city: string;
  country: string;
}

/**
 * Register a new user through the UI
 */
export async function registerUser(
  page: Page,
  user: UserCredentials,
): Promise<void> {
  await page.goto("/auth/register");

  await page.fill('input[name="email"]', user.email);
  await page.fill('input[name="password"]', user.password);
  await page.fill('input[name="name"]', user.full_name); // Note: field is "name" not "full_name"
  await page.fill('input[name="city"]', user.city);
  await page.fill('input[name="country"]', user.country);

  await page.click('button[type="submit"]');

  // Wait for login page to appear by checking for the heading
  await expect(
    page.locator('h2:has-text("Sign in to your account")'),
  ).toBeVisible({ timeout: 10000 });
}

/**
 * Login through the UI
 */
export async function login(
  page: Page,
  email: string,
  password: string,
): Promise<void> {
  await page.goto("/auth/login");

  await page.fill('input[name="email"]', email);
  await page.fill('input[name="password"]', password);

  await page.click('button[type="submit"]');

  // Wait for login to complete - check for feed page heading
  await expect(page.locator('h2:has-text("Litter Feed")')).toBeVisible({
    timeout: 10000,
  });
}

/**
 * Verify email directly via API (bypassing email system for tests)
 */
export async function verifyEmailViaAPI(email: string): Promise<void> {
  // Direct connection to backend API (Node.js context, bypasses Vite proxy)
  const baseUrl = process.env.API_BASE_URL || "http://localhost:8080";
  const response = await fetch(
    `${baseUrl}/api/test/verify-email/${encodeURIComponent(email)}`,
    {
      method: "POST",
    },
  );

  if (!response.ok) {
    throw new Error(`Failed to verify email ${email}: ${response.statusText}`);
  }

  const result = await response.json();
  if (!result.success) {
    throw new Error(`Email verification failed: ${result.message}`);
  }
}

/**
 * Register and login a verified user (combines registration + verification + login)
 */
export async function registerAndLoginVerifiedUser(
  page: Page,
  user: UserCredentials,
): Promise<void> {
  // Register
  await registerUser(page, user);

  // TODO: Verify email via API
  await verifyEmailViaAPI(user.email);

  // Login
  await login(page, user.email, user.password);
}

/**
 * Logout through the UI
 */
export async function logout(page: Page): Promise<void> {
  // Click user menu or logout button
  await page.click('button:has-text("Sign Out"), button:has-text("Logout")');

  // Wait for redirect to login
  await page.waitForURL("/auth/login", { timeout: 5000 });
}

/**
 * Check if user is logged in
 */
export async function isLoggedIn(page: Page): Promise<boolean> {
  try {
    // Try to go to a protected page
    await page.goto("/app/feed");
    await page.waitForURL("/app/feed", { timeout: 2000 });
    return true;
  } catch {
    return false;
  }
}

/**
 * Get authentication token from localStorage (for API calls)
 */
export async function getAuthToken(page: Page): Promise<string | null> {
  return await page.evaluate(() => localStorage.getItem("access_token"));
}
