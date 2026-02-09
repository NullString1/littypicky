import { Page } from '@playwright/test';

/**
 * Test data generators for e2e tests
 */

export function generateRandomEmail(): string {
	const timestamp = Date.now();
	const random = Math.random().toString(36).substring(2, 8);
	return `test-${timestamp}-${random}@littypicky-test.com`;
}

export function generateRandomUser() {
	return {
		email: generateRandomEmail(),
		password: 'TestPassword123!',
		full_name: 'Test User',
		city: 'London',
		country: 'UK',
	};
}

export function generateReportData() {
	// Central London coordinates with small random offset
	const baseLat = 51.5074;
	const baseLng = -0.1278;
	const offset = () => (Math.random() - 0.5) * 0.01; // ~500m radius

	return {
		latitude: baseLat + offset(),
		longitude: baseLng + offset(),
		description: `Test litter report - ${Date.now()}`,
	};
}

/**
 * Create a test image in base64 format (1x1 red pixel PNG)
 */
export function createTestImageBase64(): string {
	// 1x1 red pixel PNG
	return 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8DwHwAFBQIAX8jx0gAAAABJRU5ErkJggg==';
}

/**
 * Upload a file to an input element
 */
export async function uploadTestImage(page: Page, selector: string): Promise<void> {
	// Create a test image file
	const buffer = Buffer.from(
		createTestImageBase64().replace(/^data:image\/\w+;base64,/, ''),
		'base64'
	);

	// Upload the file
	await page.setInputFiles(selector, {
		name: 'test-image.png',
		mimeType: 'image/png',
		buffer,
	});
}
