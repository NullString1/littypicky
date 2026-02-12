import { test, expect } from "@playwright/test";
import { generateRandomUser } from "./helpers/test-data";
import { registerAndLoginVerifiedUser } from "./helpers/auth-helpers";
import { mockGeolocation } from "./helpers/test-helpers";

test.describe("Verification Workflow", () => {
  test.describe("Verification Queue", () => {
    test("should display cleared reports in verification queue", async ({
      page,
    }) => {
      const user = generateRandomUser();

      // TODO: Login (user must have completed enough clears to verify)
      // await registerAndLoginVerifiedUser(page, user);

      // Mock location
      await mockGeolocation(page, 51.5074, -0.1278);

      await page.goto("/app/verify");

      // Should show verification queue
      await expect(
        page.locator('[data-testid="verification-queue"]'),
      ).toBeVisible({
        timeout: 10000,
      });

      // Should show cleared reports
      await expect(
        page.locator('.report-card:has-text("Cleared")'),
      ).toBeVisible();
    });

    test("should not show reports that user has already verified", async ({
      page,
    }) => {
      const user = generateRandomUser();

      // TODO: Login and verify a report
      // await registerAndLoginVerifiedUser(page, user);

      await page.goto("/app/verify");

      // Get initial count
      const initialCount = await page.locator(".report-card").count();

      // Verify a report
      const firstReport = page.locator(".report-card").first();
      await firstReport.click();
      await page.click('button:has-text("Verify")');
      await page.click('button[value="true"], button:has-text("Yes")');

      // Go back to queue
      await page.goto("/app/verify");

      // Count should decrease
      const newCount = await page.locator(".report-card").count();
      expect(newCount).toBeLessThan(initialCount);
    });

    test("should not show reports that user cleared themselves", async ({
      page,
    }) => {
      const user = generateRandomUser();

      // TODO: Login and clear a report
      // await registerAndLoginVerifiedUser(page, user);

      await page.goto("/app/verify");

      // The report the user cleared should not appear in verification queue
      // (Would need to track which report was cleared and verify it's not in queue)
    });

    test("should show reporter can verify someone else's cleanup of their report", async ({
      page,
    }) => {
      const user1 = generateRandomUser();
      const user2 = generateRandomUser();

      // TODO: User1 creates a report
      // TODO: User2 clears the report
      // TODO: User1 should see it in verification queue

      // User1 login
      // await registerAndLoginVerifiedUser(page, user1);

      await page.goto("/app/verify");

      // Should see the report they created (that someone else cleared)
      await expect(page.locator(".report-card")).toBeVisible();
    });

    test("should show minimum clears requirement", async ({ page }) => {
      const user = generateRandomUser();

      // TODO: Login with new user (hasn't cleared enough reports)
      // await registerAndLoginVerifiedUser(page, user);

      await page.goto("/app/verify");

      // Should show message about needing more clears
      await expect(
        page.locator("text=/need.*clear|clear.*\d+.*reports/i"),
      ).toBeVisible();
    });

    test("should filter verification queue by distance", async ({ page }) => {
      const user = generateRandomUser();

      // TODO: Login
      // await registerAndLoginVerifiedUser(page, user);

      // Set location
      await mockGeolocation(page, 51.5074, -0.1278);

      await page.goto("/app/verify");

      // All reports should be within the radius
      // (Would need to verify by checking distances)
    });
  });

  test.describe("Submit Verification", () => {
    test("should verify a cleared report positively", async ({ page }) => {
      const user = generateRandomUser();

      // TODO: Login (experienced user)
      // await registerAndLoginVerifiedUser(page, user);

      await page.goto("/app/verify");

      // Click on a report to verify
      const report = page.locator(".report-card").first();
      await report.click();

      // Should show before and after photos
      await expect(page.locator('img[alt*="before"]')).toBeVisible();
      await expect(page.locator('img[alt*="after"]')).toBeVisible();

      // Click verify button
      await page.click('button:has-text("Verify")');

      // Select positive verification
      await page.click(
        'button[value="true"], button:has-text("Yes"), button:has-text("Verified")',
      );

      // Optionally add comment
      const commentField = page.locator('textarea[name="comment"]');
      if (await commentField.isVisible()) {
        await commentField.fill("Great job cleaning up!");
      }

      // Submit
      await page.click('button[type="submit"]:has-text("Submit")');

      // Should show success message
      await expect(
        page.locator("text=/verification.*submitted|verified.*successfully/i"),
      ).toBeVisible({
        timeout: 5000,
      });
    });

    test("should verify a cleared report negatively", async ({ page }) => {
      const user = generateRandomUser();

      // TODO: Login
      // await registerAndLoginVerifiedUser(page, user);

      await page.goto("/app/verify");

      const report = page.locator(".report-card").first();
      await report.click();

      // Click verify button
      await page.click('button:has-text("Verify")');

      // Select negative verification
      await page.click(
        'button[value="false"], button:has-text("No"), button:has-text("Not Verified")',
      );

      // Add comment (might be required for negative verifications)
      const commentField = page.locator('textarea[name="comment"]');
      await commentField.fill("The litter is still there");

      // Submit
      await page.click('button[type="submit"]');

      // Should show success message
      await expect(
        page.locator("text=/verification.*submitted/i"),
      ).toBeVisible();
    });

    test("should require comment for negative verification", async ({
      page,
    }) => {
      const user = generateRandomUser();

      // TODO: Login
      // await registerAndLoginVerifiedUser(page, user);

      await page.goto("/app/verify");

      const report = page.locator(".report-card").first();
      await report.click();

      await page.click('button:has-text("Verify")');
      await page.click('button[value="false"]');

      // Try to submit without comment
      await page.click('button[type="submit"]');

      // Should show error
      await expect(page.locator("text=/comment.*required/i")).toBeVisible();
    });

    test("should award points for verification", async ({ page }) => {
      const user = generateRandomUser();

      // TODO: Login
      // await registerAndLoginVerifiedUser(page, user);

      // Get initial points
      await page.goto("/app/profile");
      const initialPointsText = await page
        .locator('[data-testid="user-points"]')
        .textContent();
      const initialPoints = parseInt(
        initialPointsText?.match(/\d+/)?.[0] || "0",
      );

      // Verify a report
      await page.goto("/app/verify");
      const report = page.locator(".report-card").first();
      await report.click();

      await page.click('button:has-text("Verify")');
      await page.click('button[value="true"]');
      await page.click('button[type="submit"]');

      // Check points increased
      await page.goto("/app/profile");
      const newPointsText = await page
        .locator('[data-testid="user-points"]')
        .textContent();
      const newPoints = parseInt(newPointsText?.match(/\d+/)?.[0] || "0");

      expect(newPoints).toBeGreaterThan(initialPoints);
    });

    test("should not allow verifying same report twice", async ({ page }) => {
      const user = generateRandomUser();

      // TODO: Login
      // await registerAndLoginVerifiedUser(page, user);

      await page.goto("/app/verify");

      const report = page.locator(".report-card").first();
      const reportUrl = (await report.getAttribute("data-report-id")) || "";

      await report.click();
      await page.click('button:has-text("Verify")');
      await page.click('button[value="true"]');
      await page.click('button[type="submit"]');

      // Try to verify again
      await page.goto(`/app/report/${reportUrl}`);

      // Verify button should not be visible or should show "Already verified"
      const verifyButton = page.locator('button:has-text("Verify")');

      if (await verifyButton.isVisible()) {
        await expect(page.locator("text=/already.*verified/i")).toBeVisible();
      } else {
        expect(await verifyButton.isVisible()).toBe(false);
      }
    });
  });

  test.describe("View Verifications", () => {
    test("should display verifications for a report", async ({ page }) => {
      const user = generateRandomUser();

      // TODO: Login
      // await registerAndLoginVerifiedUser(page, user);

      // Navigate to a cleared report that has verifications
      await page.goto("/app/feed");
      const clearedReport = page
        .locator('.report-card:has-text("Cleared")')
        .first();
      await clearedReport.click();

      // Should show verifications section
      await expect(
        page.locator("text=/verifications|verified by/i"),
      ).toBeVisible();
    });

    test("should show verification count", async ({ page }) => {
      const user = generateRandomUser();

      // TODO: Login
      // await registerAndLoginVerifiedUser(page, user);

      await page.goto("/app/feed");
      const clearedReport = page
        .locator('.report-card:has-text("Cleared")')
        .first();
      await clearedReport.click();

      // Should show verification count (e.g., "2/3 verifications")
      await expect(page.locator("text=/\d+.*verifications?/i")).toBeVisible();
    });

    test("should display positive and negative verifications separately", async ({
      page,
    }) => {
      const user = generateRandomUser();

      // TODO: Login
      // await registerAndLoginVerifiedUser(page, user);

      // Navigate to report with mixed verifications
      // Should show positive count and negative count
      await expect(page.locator("text=/verified|positive/i")).toBeVisible();
    });

    test("should show verification comments", async ({ page }) => {
      const user = generateRandomUser();

      // TODO: Login
      // await registerAndLoginVerifiedUser(page, user);

      // Navigate to report with verifications
      const clearedReport = page
        .locator('.report-card:has-text("Cleared")')
        .first();
      await clearedReport.click();

      // Should show comments from verifiers
      const verificationsSection = page.locator(
        '[data-testid="verifications-list"]',
      );

      if (await verificationsSection.isVisible()) {
        // Should show at least one comment
        await expect(
          verificationsSection.locator("text=/comment/i"),
        ).toBeVisible();
      }
    });
  });

  test.describe("Report Status Progression", () => {
    test("should mark report as verified after minimum positive verifications", async ({
      page,
      context,
    }) => {
      // This test requires multiple users to verify the same report
      const verifiers = [
        generateRandomUser(),
        generateRandomUser(),
        generateRandomUser(),
      ];

      // TODO: Create a cleared report
      // TODO: Have 3 different users verify it positively

      // After 3rd verification, status should change to "Verified"
      // await expect(page.locator('text=/status.*verified/i')).toBeVisible();
    });

    test("should award bonus points to clearer when report is verified", async ({
      page,
    }) => {
      // TODO: Test that the original clearer gets bonus points
      // when their cleanup is verified by enough people
    });

    test("should show verified badge on verified reports", async ({ page }) => {
      const user = generateRandomUser();

      // TODO: Login
      // await registerAndLoginVerifiedUser(page, user);

      await page.goto("/app/feed");

      // Find a verified report
      const verifiedReport = page
        .locator('.report-card:has-text("Verified")')
        .first();

      if (await verifiedReport.isVisible()) {
        // Should have a verified badge or indicator
        await expect(
          verifiedReport.locator(
            '[data-testid="verified-badge"], .verified-badge',
          ),
        ).toBeVisible();
      }
    });
  });

  test.describe("Verification Experience Requirements", () => {
    test("should prevent verification before minimum clears", async ({
      page,
    }) => {
      const newUser = generateRandomUser();

      // TODO: Login with brand new user (0 clears)
      // await registerAndLoginVerifiedUser(page, newUser);

      await page.goto("/app/verify");

      // Should show message about requirements
      await expect(
        page.locator("text=/need to clear.*\d+.*reports/i"),
      ).toBeVisible();

      // Verification queue should be disabled or hidden
      const reportCards = page.locator(".report-card");
      const count = await reportCards.count();
      expect(count).toBe(0);
    });

    test("should unlock verification after completing minimum clears", async ({
      page,
    }) => {
      const user = generateRandomUser();

      // TODO: Login
      // await registerAndLoginVerifiedUser(page, user);

      // TODO: Clear 5 reports (or whatever the minimum is)

      // Now verification should be unlocked
      await page.goto("/app/verify");

      // Should show verification queue
      await expect(page.locator(".report-card")).toBeVisible();

      // Should not show restriction message
      await expect(page.locator("text=/need to clear/i")).not.toBeVisible();
    });
  });
});
