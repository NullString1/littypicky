import { Page, expect } from '@playwright/test';

/**
 * Custom Playwright fixtures for LittyPicky tests
 * These provide reusable setup and teardown logic
 */

/**
 * Database cleanup fixture
 * Cleans up test data after each test
 */
export async function cleanupTestData(email: string): Promise<void> {
	// Direct connection to backend API (bypasses Vite proxy)
	// This runs in Node.js test context, not browser
	const baseUrl = process.env.API_BASE_URL || 'http://localhost:8080';
	
	try {
		const response = await fetch(`${baseUrl}/api/test/cleanup`, {
			method: 'DELETE',
			headers: {
				'Content-Type': 'application/json',
			},
			body: JSON.stringify({ email }),
		});

		if (!response.ok) {
			console.warn(`Failed to cleanup test data for ${email}: ${response.statusText}`);
			return;
		}

		const result = await response.json();
		if (!result.success) {
			console.warn(`Test data cleanup warning: ${result.message}`);
		}
	} catch (error) {
		console.warn(`Error during test cleanup for ${email}:`, error);
	}
}

/**
 * API request helper with authentication
 */
export async function apiRequest(
	page: Page,
	endpoint: string,
	options: RequestInit = {}
): Promise<Response> {
	const token = await page.evaluate(() => localStorage.getItem('access_token'));
	
	const headers = new Headers(options.headers || {});
	if (token) {
		headers.set('Authorization', `Bearer ${token}`);
	}
	headers.set('Content-Type', 'application/json');
	
	// Direct connection to backend (Node.js context)
	return fetch(`http://localhost:8080${endpoint}`, {
		...options,
		headers,
	});
}

/**
 * Wait for API call to complete
 */
export async function waitForApiResponse(
	page: Page,
	urlPattern: string | RegExp,
	callback: () => Promise<void>
): Promise<any> {
	const responsePromise = page.waitForResponse(
		response => {
			const url = response.url();
			if (typeof urlPattern === 'string') {
				return url.includes(urlPattern);
			}
			return urlPattern.test(url);
		},
		{ timeout: 10000 }
	);
	
	await callback();
	
	const response = await responsePromise;
	return response.json();
}

/**
 * Mock geolocation to a specific location
 */
export async function mockGeolocation(
	page: Page,
	latitude: number,
	longitude: number
): Promise<void> {
	await page.context().grantPermissions(['geolocation']);
	await page.context().setGeolocation({ latitude, longitude });
}

/**
 * Wait for element to be visible with custom timeout
 */
export async function waitForElement(
	page: Page,
	selector: string,
	timeout: number = 5000
): Promise<void> {
	await page.waitForSelector(selector, { state: 'visible', timeout });
}

/**
 * Check if element exists (without throwing)
 */
export async function elementExists(page: Page, selector: string): Promise<boolean> {
	return (await page.locator(selector).count()) > 0;
}

/**
 * Get text content from element
 */
export async function getTextContent(page: Page, selector: string): Promise<string> {
	const element = await page.locator(selector);
	return (await element.textContent()) || '';
}

/**
 * Take a screenshot with custom name
 */
export async function takeScreenshot(page: Page, name: string): Promise<void> {
	await page.screenshot({ path: `test-results/screenshots/${name}.png`, fullPage: true });
}
