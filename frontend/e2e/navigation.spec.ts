import { test, expect } from '@playwright/test';

/**
 * Navigation and Page Loading Tests
 *
 * Tests all main navigation links and verifies each page loads correctly
 * with expected content.
 */
test.describe('Navigation and Page Loading', () => {
  test.beforeEach(async ({ page }) => {
    // Start from the dashboard
    await page.goto('/');
    await expect(page).toHaveURL('/');
  });

  test('Dashboard loads with correct content', async ({ page }) => {
    // Check for main heading
    await expect(page.locator('h1')).toContainText('Dashboard');

    // Check for welcome message
    await expect(page.getByText('Welcome to Product-FARM')).toBeVisible();

    // Check quick actions section
    await expect(page.getByText('Quick Actions')).toBeVisible();
  });

  test('Products page loads correctly', async ({ page }) => {
    await page.getByRole('link', { name: 'Products' }).click();
    await expect(page).toHaveURL('/products');

    // Check main heading
    await expect(page.locator('h1')).toContainText('Products');

    // Check for product stats section
    await expect(page.getByText('Total')).toBeVisible();

    // Check New Product button
    await expect(page.getByRole('button', { name: 'New Product' })).toBeVisible();
  });

  test('Datatypes page loads correctly', async ({ page }) => {
    await page.getByRole('link', { name: 'Datatypes' }).click();
    await expect(page).toHaveURL('/datatypes');

    // Check main heading
    await expect(page.locator('h1')).toContainText('Datatypes');

    // Check for stats
    await expect(page.getByText('Total Types')).toBeVisible();

    // Check Add button
    await expect(page.getByRole('button', { name: 'Add Datatype' })).toBeVisible();
  });

  test('Enumerations page loads correctly', async ({ page }) => {
    await page.getByRole('link', { name: 'Enumerations' }).click();
    await expect(page).toHaveURL('/enumerations');

    // Check main heading
    await expect(page.locator('h1')).toContainText('Enumerations');

    // Check for stats
    await expect(page.getByText('Total Enumerations')).toBeVisible();

    // Check Add button
    await expect(page.getByRole('button', { name: 'Add Enumeration' })).toBeVisible();
  });

  test('Attributes page loads correctly', async ({ page }) => {
    await page.getByRole('link', { name: 'Attributes', exact: true }).click();
    await expect(page).toHaveURL('/attributes');

    // Check main heading
    await expect(page.locator('h1')).toContainText('Attributes');

    // Check for product selector on Attributes page
    await expect(page.getByText('Select a product').first()).toBeVisible({ timeout: 10000 });
  });

  test('Functions page loads correctly', async ({ page }) => {
    await page.getByRole('link', { name: 'Functions' }).click();
    await expect(page).toHaveURL('/functionalities');

    // Check main heading - nav shows "Functions", h1 also shows "Functions"
    await expect(page.locator('h1')).toContainText('Functions');
  });

  test('Rules page loads with canvas', async ({ page }) => {
    await page.getByRole('link', { name: 'Rules', exact: true }).click();
    await expect(page).toHaveURL('/rules');

    // Check main heading
    await expect(page.locator('h1')).toContainText('Rules');

    // Check for toolbar buttons - Rules page auto-selects a product
    await expect(page.getByRole('button', { name: 'New Rule' })).toBeVisible({ timeout: 10000 });
  });

  test('Settings page loads correctly', async ({ page }) => {
    await page.getByRole('link', { name: 'Settings' }).click();
    await expect(page).toHaveURL('/settings');

    // Check that the page loads (settings may have various content)
    await expect(page.locator('main')).toBeVisible();
  });

  test('Navigation between pages works correctly', async ({ page }) => {
    // Navigate through all pages - use exact match to avoid strict mode violations
    await page.getByRole('link', { name: 'Products', exact: true }).click();
    await expect(page).toHaveURL('/products');

    await page.getByRole('link', { name: 'Rules', exact: true }).click();
    await expect(page).toHaveURL('/rules');

    await page.getByRole('link', { name: 'Dashboard', exact: true }).click();
    await expect(page).toHaveURL('/');
  });
});
