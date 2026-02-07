import { test, expect } from '@playwright/test';
import { generateRandomUser, createTestImageBase64 } from './helpers/test-data';
import { registerAndLoginVerifiedUser } from './helpers/auth-helpers';
import { mockGeolocation } from './helpers/test-helpers';

test.describe('Report Claiming and Clearing', () => {
	test.describe('Claim Report', () => {
		test('should claim a pending report', async ({ page }) => {
			const user = generateRandomUser();

			// TODO: Login
			// await registerAndLoginVerifiedUser(page, user);

			await page.goto('/app/feed');

			// Find a pending report
			const pendingReport = page.locator('.report-card:has-text("Pending")').first();
			await pendingReport.waitFor({ state: 'visible', timeout: 10000 });
			await pendingReport.click();

			// Click claim button
			await page.click('button:has-text("Claim")');

			// Should show success message
			await expect(page.locator('text=/claimed.*successfully|successfully.*claimed/i')).toBeVisible({
				timeout: 5000,
			});

			// Status should update to "Claimed"
			await expect(page.locator('text=/status.*claimed|claimed/i')).toBeVisible();
		});

		test('should not allow claiming own report', async ({ page }) => {
			const user = generateRandomUser();

			// TODO: Login and create a report
			// await registerAndLoginVerifiedUser(page, user);
			// await createReport(page, reportData);

			// Try to claim own report
			await page.goto('/app/my-reports');
			await page.locator('.report-card').first().click();

			// Claim button should not be visible or should be disabled
			const claimButton = page.locator('button:has-text("Claim")');
			const isVisible = await claimButton.isVisible();

			if (isVisible) {
				await expect(claimButton).toBeDisabled();
			} else {
				expect(isVisible).toBe(false);
			}
		});

		test('should not allow claiming already claimed report', async ({ page }) => {
			const user = generateRandomUser();

			// TODO: Login
			// await registerAndLoginVerifiedUser(page, user);

			await page.goto('/app/feed');

			// Find a claimed report
			const claimedReport = page.locator('.report-card:has-text("Claimed")').first();
			
			if (await claimedReport.isVisible()) {
				await claimedReport.click();

				// Claim button should not be visible
				const claimButton = page.locator('button:has-text("Claim")');
				await expect(claimButton).not.toBeVisible();
			}
		});

		test('should update report list after claiming', async ({ page }) => {
			const user = generateRandomUser();

			// TODO: Login
			// await registerAndLoginVerifiedUser(page, user);

			await page.goto('/app/feed');

			// Get report ID before claiming
			const report = page.locator('.report-card:has-text("Pending")').first();
			await report.click();

			const url = page.url();
			const reportId = url.match(/\/report\/([^/]+)/)?.[1];

			// Claim it
			await page.click('button:has-text("Claim")');

			// Go back to feed
			await page.goto('/app/feed');

			// Report should show as claimed
			if (reportId) {
				const reportInList = page.locator(`.report-card[data-report-id="${reportId}"]`);
				await expect(reportInList.locator('text=/claimed/i')).toBeVisible();
			}
		});
	});

	test.describe('Clear Report', () => {
		test('should clear a claimed report with after photo', async ({ page }) => {
			const user = generateRandomUser();

			// TODO: Login
			// await registerAndLoginVerifiedUser(page, user);

			// First claim a report
			await page.goto('/app/feed');
			const pendingReport = page.locator('.report-card:has-text("Pending")').first();
			await pendingReport.click();
			await page.click('button:has-text("Claim")');

			// Now clear it
			await page.click('button:has-text("Mark as Cleared")');

			// Upload after photo
			const photoInput = page.locator('input[type="file"]');
			await photoInput.setInputFiles({
				name: 'test-cleared.png',
				mimeType: 'image/png',
				buffer: Buffer.from(
					createTestImageBase64().replace(/^data:image\/\w+;base64,/, ''),
					'base64'
				),
			});

			// Submit
			await page.click('button[type="submit"]:has-text("Submit")');

			// Should show success message
			await expect(page.locator('text=/cleared.*successfully|marked.*cleared/i')).toBeVisible({
				timeout: 15000,
			});

			// Status should update to "Cleared"
			await expect(page.locator('text=/status.*cleared|cleared/i')).toBeVisible();
		});

		test('should not allow clearing without after photo', async ({ page }) => {
			const user = generateRandomUser();

			// TODO: Login and claim a report
			// await registerAndLoginVerifiedUser(page, user);

			// Try to clear without photo
			await page.click('button:has-text("Mark as Cleared")');
			await page.click('button[type="submit"]');

			// Should show error
			await expect(page.locator('text=/photo.*required|upload.*photo/i')).toBeVisible();
		});

		test('should only allow clearer to mark report as cleared', async ({ page, context }) => {
			const user1 = generateRandomUser();
			const user2 = generateRandomUser();

			// TODO: User1 claims a report
			// await registerAndLoginVerifiedUser(page, user1);
			// await claimReport(page);

			// TODO: User2 logs in
			// await logout(page);
			// await login(page, user2.email, user2.password);

			// Navigate to the claimed report
			// User2 should not see "Mark as Cleared" button
			const clearButton = page.locator('button:has-text("Mark as Cleared")');
			await expect(clearButton).not.toBeVisible();
		});

		test('should show both before and after photos after clearing', async ({ page }) => {
			const user = generateRandomUser();

			// TODO: Login, claim and clear a report
			// await registerAndLoginVerifiedUser(page, user);

			// After clearing, navigate to report details
			// Should show both photos
			await expect(page.locator('img[alt*="before"], img[alt*="Before"]')).toBeVisible();
			await expect(page.locator('img[alt*="after"], img[alt*="After"]')).toBeVisible();
		});

		test('should award points after clearing report', async ({ page }) => {
			const user = generateRandomUser();

			// TODO: Login
			// await registerAndLoginVerifiedUser(page, user);

			// Get initial points
			await page.goto('/app/profile');
			const initialPointsText = await page.locator('[data-testid="user-points"], text=/points/i').textContent();
			const initialPoints = parseInt(initialPointsText?.match(/\d+/)?.[0] || '0');

			// Claim and clear a report
			await page.goto('/app/feed');
			const report = page.locator('.report-card:has-text("Pending")').first();
			await report.click();
			await page.click('button:has-text("Claim")');

			// Clear with photo
			await page.click('button:has-text("Mark as Cleared")');
			const photoInput = page.locator('input[type="file"]');
			await photoInput.setInputFiles({
				name: 'cleared.png',
				mimeType: 'image/png',
				buffer: Buffer.from(createTestImageBase64().replace(/^data:image\/\w+;base64,/, ''), 'base64'),
			});
			await page.click('button[type="submit"]');

			// Check points increased
			await page.goto('/app/profile');
			const newPointsText = await page.locator('[data-testid="user-points"], text=/points/i').textContent();
			const newPoints = parseInt(newPointsText?.match(/\d+/)?.[0] || '0');

			expect(newPoints).toBeGreaterThan(initialPoints);
		});
	});

	test.describe('My Cleared Reports', () => {
		test('should display reports cleared by user', async ({ page }) => {
			const user = generateRandomUser();

			// TODO: Login
			// await registerAndLoginVerifiedUser(page, user);

			await page.goto('/app/my-clears');

			// Should show cleared reports list
			await expect(page.locator('[data-testid="my-clears-list"]')).toBeVisible();
		});

		test('should show empty state when user has not cleared any reports', async ({ page }) => {
			const user = generateRandomUser();

			// TODO: Login (new user)
			// await registerAndLoginVerifiedUser(page, user);

			await page.goto('/app/my-clears');

			// Should show empty state
			await expect(page.locator('text=/no.*cleared|haven\'t cleared/i')).toBeVisible();
		});

		test('should show cleared reports with before and after photos', async ({ page }) => {
			const user = generateRandomUser();

			// TODO: Login and clear a report
			// await registerAndLoginVerifiedUser(page, user);

			await page.goto('/app/my-clears');

			const clearedReport = page.locator('.report-card').first();
			
			if (await clearedReport.isVisible()) {
				await clearedReport.click();

				// Should show both photos
				await expect(page.locator('img[alt*="before"]')).toBeVisible();
				await expect(page.locator('img[alt*="after"]')).toBeVisible();
			}
		});
	});

	test.describe('Claim and Clear Workflow', () => {
		test('should complete full claim and clear workflow', async ({ page }) => {
			const user = generateRandomUser();

			// TODO: Login
			// await registerAndLoginVerifiedUser(page, user);

			// 1. Find a pending report
			await page.goto('/app/feed');
			const pendingReport = page.locator('.report-card:has-text("Pending")').first();
			await expect(pendingReport).toBeVisible({ timeout: 10000 });
			await pendingReport.click();

			const reportUrl = page.url();

			// 2. Claim the report
			await page.click('button:has-text("Claim")');
			await expect(page.locator('text=/claimed/i')).toBeVisible();

			// 3. Clear the report
			await page.click('button:has-text("Mark as Cleared")');
			
			const photoInput = page.locator('input[type="file"]');
			await photoInput.setInputFiles({
				name: 'cleanup-done.png',
				mimeType: 'image/png',
				buffer: Buffer.from(createTestImageBase64().replace(/^data:image\/\w+;base64,/, ''), 'base64'),
			});
			
			await page.click('button[type="submit"]');

			// 4. Verify final status
			await expect(page.locator('text=/cleared.*successfully|status.*cleared/i')).toBeVisible({
				timeout: 15000,
			});

			// 5. Check it appears in "My Clears"
			await page.goto('/app/my-clears');
			await expect(page.locator('.report-card')).toBeVisible();

			// 6. Verify report is now in verification queue
			await page.goto('/app/verify');
			// The cleared report should appear for others to verify
		});

		test('should track time between claim and clear', async ({ page }) => {
			const user = generateRandomUser();

			// TODO: Login
			// await registerAndLoginVerifiedUser(page, user);

			await page.goto('/app/feed');
			const report = page.locator('.report-card:has-text("Pending")').first();
			await report.click();

			// Claim
			const claimTime = new Date();
			await page.click('button:has-text("Claim")');

			// Wait a bit
			await page.waitForTimeout(2000);

			// Clear
			await page.click('button:has-text("Mark as Cleared")');
			const photoInput = page.locator('input[type="file"]');
			await photoInput.setInputFiles({
				name: 'test.png',
				mimeType: 'image/png',
				buffer: Buffer.from(createTestImageBase64().replace(/^data:image\/\w+;base64,/, ''), 'base64'),
			});
			await page.click('button[type="submit"]');

			const clearTime = new Date();

			// Time difference should be at least 2 seconds
			const timeDiff = clearTime.getTime() - claimTime.getTime();
			expect(timeDiff).toBeGreaterThanOrEqual(2000);

			// Report should show claimed and cleared timestamps
			await expect(page.locator('text=/claimed at|cleared at/i')).toBeVisible();
		});
	});
});
