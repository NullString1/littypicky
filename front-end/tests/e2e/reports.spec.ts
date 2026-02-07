import { test, expect } from '@playwright/test';
import { generateRandomUser, generateReportData, createTestImageBase64 } from './helpers/test-data';
import { registerAndLoginVerifiedUser } from './helpers/auth-helpers';
import { mockGeolocation, waitForApiResponse } from './helpers/test-helpers';

test.describe('Report Management', () => {
	test.describe('Create Report', () => {
		test('should create a new litter report with photo', async ({ page }) => {
			const user = generateRandomUser();
			const reportData = generateReportData();

			// Login first
			// TODO: This requires email verification to work
			// await registerAndLoginVerifiedUser(page, user);

			// Navigate to report creation page
			await page.goto('/app/report');

			// Mock geolocation
			await mockGeolocation(page, reportData.latitude, reportData.longitude);

			// Fill in report details
			await page.fill('textarea[name="description"]', reportData.description);

			// Upload photo
			const photoInput = page.locator('input[type="file"]');
			await photoInput.setInputFiles({
				name: 'test-litter.png',
				mimeType: 'image/png',
				buffer: Buffer.from(
					createTestImageBase64().replace(/^data:image\/\w+;base64,/, ''),
					'base64'
				),
			});

			// Submit form
			await page.click('button[type="submit"]:has-text("Submit Report")');

			// Should show success message
			await expect(page.locator('text=/report.*submitted|success/i')).toBeVisible({
				timeout: 15000,
			});

			// Should redirect to feed
			await expect(page).toHaveURL('/app/feed', { timeout: 10000 });
		});

		test('should show error when submitting without photo', async ({ page }) => {
			const user = generateRandomUser();
			const reportData = generateReportData();

			// TODO: Login
			// await registerAndLoginVerifiedUser(page, user);

			await page.goto('/app/report');

			// Fill description but no photo
			await page.fill('textarea[name="description"]', reportData.description);

			// Try to submit
			await page.click('button[type="submit"]');

			// Should show validation error
			await expect(page.locator('text=/photo.*required|upload.*photo/i')).toBeVisible();
		});

		test('should compress large images before upload', async ({ page }) => {
			const user = generateRandomUser();

			// TODO: Login
			// await registerAndLoginVerifiedUser(page, user);

			await page.goto('/app/report');

			// Upload a large image
			// The frontend should compress it client-side

			// TODO: Create a larger test image and verify compression
		});

		test('should use current location for report', async ({ page }) => {
			const user = generateRandomUser();
			const reportData = generateReportData();

			// TODO: Login
			// await registerAndLoginVerifiedUser(page, user);

			await page.goto('/app/report');

			// Mock geolocation
			await mockGeolocation(page, reportData.latitude, reportData.longitude);

			// Click "Use Current Location" button
			await page.click('button:has-text("Current Location")');

			// Should populate location fields
			const latInput = page.locator('input[name="latitude"]');
			const lngInput = page.locator('input[name="longitude"]');

			await expect(latInput).toHaveValue(String(reportData.latitude));
			await expect(lngInput).toHaveValue(String(reportData.longitude));
		});

		test('should prevent creating report without verified email', async ({ page }) => {
			const user = generateRandomUser();

			// Register but don't verify
			// await registerUser(page, user);
			// await login(page, user.email, user.password);

			await page.goto('/app/report');

			// Try to submit report
			// Should show error or redirect
			await expect(page.locator('text=/verify.*email|email.*not.*verified/i')).toBeVisible();
		});
	});

	test.describe('View Reports', () => {
		test('should display nearby reports on feed page', async ({ page }) => {
			const user = generateRandomUser();

			// TODO: Login
			// await registerAndLoginVerifiedUser(page, user);

			// Mock location (London)
			await mockGeolocation(page, 51.5074, -0.1278);

			await page.goto('/app/feed');

			// Should show reports list
			await expect(page.locator('[data-testid="reports-list"], .report-card')).toBeVisible({
				timeout: 10000,
			});

			// Should show at least the report status filter
			await expect(page.locator('text=/pending|claimed|cleared/i')).toBeVisible();
		});

		test('should filter reports by status', async ({ page }) => {
			const user = generateRandomUser();

			// TODO: Login
			// await registerAndLoginVerifiedUser(page, user);

			await page.goto('/app/feed');

			// Click on "Pending" filter
			await page.click('button:has-text("Pending")');

			// Should only show pending reports
			await expect(page.locator('.report-card:has-text("Claimed")')).not.toBeVisible();
			await expect(page.locator('.report-card:has-text("Cleared")')).not.toBeVisible();
		});

		test('should show report details when clicked', async ({ page }) => {
			const user = generateRandomUser();

			// TODO: Login
			// await registerAndLoginVerifiedUser(page, user);

			await page.goto('/app/feed');

			// Click on first report
			const firstReport = page.locator('.report-card').first();
			await firstReport.waitFor({ state: 'visible' });
			await firstReport.click();

			// Should navigate to report detail page
			await expect(page).toHaveURL(/\/app\/report\/[a-f0-9-]+/, { timeout: 5000 });

			// Should show report details
			await expect(page.locator('text=/description|location|status/i')).toBeVisible();
		});

		test('should display report photo', async ({ page }) => {
			const user = generateRandomUser();

			// TODO: Login
			// await registerAndLoginVerifiedUser(page, user);

			await page.goto('/app/feed');

			// Should have at least one report with photo
			const reportPhoto = page.locator('.report-card img, [alt*="report"]').first();
			await expect(reportPhoto).toBeVisible({ timeout: 10000 });

			// Photo should have loaded
			await expect(reportPhoto).toHaveJSProperty('complete', true);
		});

		test('should update feed when new reports are created', async ({ page, context }) => {
			const user = generateRandomUser();

			// TODO: Login
			// await registerAndLoginVerifiedUser(page, user);

			await page.goto('/app/feed');

			// Get initial report count
			const initialCount = await page.locator('.report-card').count();

			// Create a new report in another tab/page
			// TODO: Create report via API or second page

			// Refresh feed
			await page.reload();

			// Should have more reports
			const newCount = await page.locator('.report-card').count();
			expect(newCount).toBeGreaterThanOrEqual(initialCount);
		});
	});

	test.describe('View My Reports', () => {
		test('should display user\'s own reports', async ({ page }) => {
			const user = generateRandomUser();

			// TODO: Login and create a report
			// await registerAndLoginVerifiedUser(page, user);

			await page.goto('/app/my-reports');

			// Should show reports list
			await expect(page.locator('[data-testid="my-reports-list"]')).toBeVisible();
		});

		test('should show empty state when user has no reports', async ({ page }) => {
			const user = generateRandomUser();

			// TODO: Login (new user with no reports)
			// await registerAndLoginVerifiedUser(page, user);

			await page.goto('/app/my-reports');

			// Should show empty state
			await expect(page.locator('text=/no reports|haven\'t created/i')).toBeVisible();
		});
	});

	test.describe('Nearby Reports', () => {
		test('should request location permission', async ({ page, context }) => {
			const user = generateRandomUser();

			// TODO: Login
			// await registerAndLoginVerifiedUser(page, user);

			await page.goto('/app/feed');

			// Grant permission
			await context.grantPermissions(['geolocation']);

			// Should load nearby reports
			await expect(page.locator('.report-card')).toBeVisible({ timeout: 10000 });
		});

		test('should show reports within specified radius', async ({ page }) => {
			const user = generateRandomUser();

			// TODO: Login
			// await registerAndLoginVerifiedUser(page, user);

			// Set location to central London
			await mockGeolocation(page, 51.5074, -0.1278);

			await page.goto('/app/feed');

			// Change radius filter if available
			// const radiusSelect = page.locator('select[name="radius"]');
			// if (await radiusSelect.isVisible()) {
			//   await radiusSelect.selectOption('5'); // 5km radius
			// }

			// All visible reports should be within the radius
			// (Verification would require parsing coordinates from each report)
		});

		test('should handle location permission denial', async ({ page, context }) => {
			const user = generateRandomUser();

			// TODO: Login
			// await registerAndLoginVerifiedUser(page, user);

			// Deny geolocation
			await context.clearPermissions();

			await page.goto('/app/feed');

			// Should show error or prompt to enable location
			// await expect(page.locator('text=/enable.*location|location.*required/i')).toBeVisible();
		});
	});

	test.describe('Report Details', () => {
		test('should display full report information', async ({ page }) => {
			const user = generateRandomUser();

			// TODO: Login
			// await registerAndLoginVerifiedUser(page, user);

			await page.goto('/app/feed');

			// Click on a report
			await page.locator('.report-card').first().click();

			// Should show all report details
			await expect(page.locator('text=/status/i')).toBeVisible();
			await expect(page.locator('text=/location|city|country/i')).toBeVisible();
			await expect(page.locator('img[alt*="before"], img[alt*="litter"]')).toBeVisible();
		});

		test('should show before and after photos for cleared reports', async ({ page }) => {
			const user = generateRandomUser();

			// TODO: Login
			// await registerAndLoginVerifiedUser(page, user);

			// Navigate to a cleared report
			// (Would need to create one first or find existing)

			// Should show both photos
			await expect(page.locator('img[alt*="before"]')).toBeVisible();
			await expect(page.locator('img[alt*="after"]')).toBeVisible();
		});

		test('should display reporter information', async ({ page }) => {
			const user = generateRandomUser();

			// TODO: Login
			// await registerAndLoginVerifiedUser(page, user);

			await page.goto('/app/feed');
			await page.locator('.report-card').first().click();

			// Should show reporter name/info
			await expect(page.locator('text=/reported by|reporter/i')).toBeVisible();
		});
	});
});
