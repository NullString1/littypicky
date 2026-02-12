import { test, expect } from "@playwright/test";
import { generateRandomUser } from "./helpers/test-data";
import {
  registerUser,
  login,
  logout,
  isLoggedIn,
  verifyEmailViaAPI,
  registerAndLoginVerifiedUser,
} from "./helpers/auth-helpers";
import { cleanupTestData } from "./helpers/test-helpers";

test.describe("Authentication", () => {
  test.describe("User Registration", () => {
    test("should register a new user successfully", async ({ page }) => {
      const user = generateRandomUser();

      await page.goto("/auth/register");

      // Fill registration form
      await page.fill('input[name="email"]', user.email);
      await page.fill('input[name="password"]', user.password);
      await page.fill('input[name="name"]', user.full_name);
      await page.fill('input[name="city"]', user.city);
      await page.fill('input[name="country"]', user.country);

      // Submit form
      await page.click('button[type="submit"]');

      // Should redirect to login page - check by content instead of URL
      await expect(
        page.locator('h2:has-text("Sign in to your account")'),
      ).toBeVisible({ timeout: 10000 });

      // Cleanup
      await cleanupTestData(user.email);
    });

    test("should show validation error for invalid email", async ({ page }) => {
      await page.goto("/auth/register");

      await page.fill('input[name="email"]', "invalid-email");
      await page.fill('input[name="password"]', "password123");
      await page.fill('input[name="name"]', "Test User");
      await page.fill('input[name="city"]', "New York");
      await page.fill('input[name="country"]', "USA");
      await page.click('button[type="submit"]');

      // Should show error message
      await expect(page.locator("text=/invalid.*email/i")).toBeVisible();
    });

    test("should show validation error for weak password", async ({ page }) => {
      const user = generateRandomUser();

      await page.goto("/auth/register");

      await page.fill('input[name="email"]', user.email);
      await page.fill('input[name="password"]', "123"); // Too short
      await page.fill('input[name="name"]', user.full_name);
      await page.fill('input[name="city"]', user.city);
      await page.fill('input[name="country"]', user.country);
      await page.click('button[type="submit"]');

      // Should show error message
      await expect(page.locator("text=/password.*short|weak/i")).toBeVisible();
    });

    test("should prevent duplicate email registration", async ({ page }) => {
      const user = generateRandomUser();

      // Register first time
      await registerUser(page, user);

      // Try to register again with same email
      await page.goto("/auth/register");
      await page.fill('input[name="email"]', user.email);
      await page.fill('input[name="password"]', "DifferentPassword123!");
      await page.fill('input[name="name"]', "Different Name");
      await page.fill('input[name="city"]', "Paris");
      await page.fill('input[name="country"]', "France");
      await page.click('button[type="submit"]');

      // Should show error
      await expect(
        page.locator("text=/email.*already.*exists|already.*registered/i"),
      ).toBeVisible();

      // Cleanup
      await cleanupTestData(user.email);
    });
  });

  test.describe("User Login", () => {
    test("should login with valid credentials", async ({ page }) => {
      const user = generateRandomUser();

      // Register the user
      await registerUser(page, user);

      // Verify email via API
      await verifyEmailViaAPI(user.email);

      // Then login
      await page.goto("/auth/login");
      await page.fill('input[name="email"]', user.email);
      await page.fill('input[name="password"]', user.password);
      await page.click('button[type="submit"]');

      // Should redirect to app - check for feed page heading
      await expect(page.locator('h2:has-text("Litter Feed")')).toBeVisible({
        timeout: 10000,
      });

      // Cleanup
      await cleanupTestData(user.email);
    });

    test("should show error for invalid credentials", async ({ page }) => {
      await page.goto("/auth/login");

      await page.fill('input[name="email"]', "nonexistent@example.com");
      await page.fill('input[name="password"]', "wrongpassword");
      await page.click('button[type="submit"]');

      // Should show error message
      await expect(
        page.locator("text=/invalid.*credentials|email.*password.*incorrect/i"),
      ).toBeVisible();
    });

    test("should show error for unverified email", async ({ page }) => {
      const user = generateRandomUser();

      // Register but don't verify
      await registerUser(page, user);

      // Try to login
      await page.goto("/auth/login");
      await page.fill('input[name="email"]', user.email);
      await page.fill('input[name="password"]', user.password);
      await page.click('button[type="submit"]');

      // Should show error about unverified email
      await expect(
        page.locator("text=/verify.*email|email.*not.*verified/i"),
      ).toBeVisible();

      // Cleanup
      await cleanupTestData(user.email);
    });

    test("should persist login after page refresh", async ({
      page,
      context,
    }) => {
      const user = generateRandomUser();

      // Register, verify, and login
      await registerAndLoginVerifiedUser(page, user);

      // Check if logged in - look for feed heading
      await expect(page.locator('h2:has-text("Litter Feed")')).toBeVisible({
        timeout: 10000,
      });

      // Refresh the page
      await page.reload();

      // Should still show feed heading
      await expect(page.locator('h2:has-text("Litter Feed")')).toBeVisible({
        timeout: 10000,
      });

      // Cleanup
      await cleanupTestData(user.email);
    });
  });

  test.describe("User Logout", () => {
    test("should logout successfully", async ({ page }) => {
      const user = generateRandomUser();

      // Login first
      await registerAndLoginVerifiedUser(page, user);

      // Should be on feed page - check for heading
      await expect(page.locator('h2:has-text("Litter Feed")')).toBeVisible({
        timeout: 10000,
      });

      // Logout
      await page.click('button:has-text("Sign Out")');

      // Should redirect to login page - check for heading
      await expect(
        page.locator('h2:has-text("Sign in to your account")'),
      ).toBeVisible({ timeout: 10000 });

      // Should not be able to access protected pages
      await page.goto("/app/feed");
      await expect(
        page.locator('h2:has-text("Sign in to your account")'),
      ).toBeVisible({ timeout: 10000 });

      // Cleanup
      await cleanupTestData(user.email);
    });
  });

  test.describe("Email Verification", () => {
    test("should show success message after registration", async ({ page }) => {
      const user = generateRandomUser();

      // Register user - the helper will handle navigation
      await registerUser(page, user);

      // Should be on login page after registration - check for heading
      await expect(
        page.locator('h2:has-text("Sign in to your account")'),
      ).toBeVisible({ timeout: 10000 });

      // Cleanup
      await cleanupTestData(user.email);
    });

    test.skip("should allow resending verification email", async ({ page }) => {
      // TODO: This requires a dedicated verification pending page
      // Currently app shows alert and redirects to login
    });

    test.skip("should verify email with valid token", async ({ page }) => {
      // TODO: Implement this test
      // Requires getting verification token from email or database
    });

    test.skip("should show error for invalid verification token", async ({
      page,
    }) => {
      // TODO: Implement this test
    });
  });

  test.describe("Password Reset", () => {
    test("should request password reset", async ({ page }) => {
      const user = generateRandomUser();

      // Register user first
      await registerUser(page, user);

      // Go to forgot password page
      await page.goto("/auth/forgot-password");

      await page.fill('input[name="email"]', user.email);
      await page.click('button[type="submit"]');

      // Should show success message
      await expect(
        page.locator("text=/password.*reset.*sent|check.*email/i"),
      ).toBeVisible();

      // Cleanup
      await cleanupTestData(user.email);
    });

    test.skip("should reset password with valid token", async ({ page }) => {
      // TODO: Implement this test
      // Requires getting reset token from email or database
    });

    test.skip("should show error for expired reset token", async ({ page }) => {
      // TODO: Implement this test
    });
  });

  test.describe("Protected Routes", () => {
    test("should redirect to login when accessing protected route while logged out", async ({
      page,
    }) => {
      await page.goto("/app/feed");

      // Should redirect to login - check for heading
      await expect(
        page.locator('h2:has-text("Sign in to your account")'),
      ).toBeVisible({ timeout: 10000 });
    });

    test("should redirect to login when accessing profile while logged out", async ({
      page,
    }) => {
      await page.goto("/app/profile");

      // Should redirect to login - check for heading
      await expect(
        page.locator('h2:has-text("Sign in to your account")'),
      ).toBeVisible({ timeout: 10000 });
    });
  });
});
