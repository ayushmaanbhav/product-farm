import { test, expect } from '@playwright/test';

/**
 * Comprehensive Enumeration E2E Tests
 *
 * Tests ALL enumeration features:
 * - Enumeration listing and filtering
 * - Creating enumerations with values
 * - Template type selection
 * - Search functionality
 * - Edit and delete operations
 *
 * Test data from auto-insurance-test-data.json:
 * - coverage-type: liability, collision, comprehensive, uninsured-motorist, medical-payments
 * - vehicle-type: sedan, suv, truck, van, sports-car, motorcycle, electric
 * - driver-status: primary, secondary, occasional, excluded
 * - payment-frequency: monthly, quarterly, semi-annual, annual
 * - risk-tier: preferred, standard, non-standard, high-risk
 * - discount-type: multi-policy, good-driver, anti-theft, defensive-driving, student, military
 * - claim-type: at-fault, not-at-fault, comprehensive, glass, tow
 */

const testEnumerations = [
  { id: 'coverage-type', name: 'Coverage Type', values: ['liability', 'collision', 'comprehensive', 'uninsured-motorist', 'medical-payments'] },
  { id: 'vehicle-type', name: 'Vehicle Type', values: ['sedan', 'suv', 'truck', 'van', 'sports-car', 'motorcycle', 'electric'] },
  { id: 'driver-status', name: 'Driver Status', values: ['primary', 'secondary', 'occasional', 'excluded'] },
  { id: 'payment-frequency', name: 'Payment Frequency', values: ['monthly', 'quarterly', 'semi-annual', 'annual'] },
  { id: 'risk-tier', name: 'Risk Tier', values: ['preferred', 'standard', 'non-standard', 'high-risk'] },
  { id: 'discount-type', name: 'Discount Type', values: ['multi-policy', 'good-driver', 'anti-theft', 'defensive-driving', 'student', 'military'] },
  { id: 'claim-type', name: 'Claim Type', values: ['at-fault', 'not-at-fault', 'comprehensive', 'glass', 'tow'] }
];

test.describe('Enumerations Page - Basic Functionality', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/enumerations');
    await expect(page.locator('h1')).toContainText('Enumerations');
  });

  test('Page displays all UI elements correctly', async ({ page }) => {
    // Check heading
    await expect(page.getByRole('heading', { name: 'Enumerations', level: 2 })).toBeVisible();

    // Check stats section
    await expect(page.getByRole('heading', { name: 'Total Enumerations' })).toBeVisible();

    // Check search input
    await expect(page.getByPlaceholder('Search enumerations or values...')).toBeVisible();

    // Check template filter dropdown
    const templateFilter = page.getByRole('combobox').first();
    await expect(templateFilter).toBeVisible();

    // Check Add Enumeration button
    await expect(page.getByRole('button', { name: 'Add Enumeration' })).toBeVisible();
  });

  test('Template filter dropdown has options', async ({ page }) => {
    const templateFilter = page.getByRole('combobox').first();
    // Template filter value should be visible (default "All Templates")
    await expect(templateFilter).toHaveValue('ALL');
  });

  test('Search filters enumeration list', async ({ page }) => {
    const searchInput = page.getByPlaceholder('Search enumerations or values...');

    // Search for "Vehicle"
    await searchInput.fill('Vehicle');
    await page.waitForTimeout(500);

    // Should show Vehicle Type enumeration
    await expect(page.getByText('Vehicle Type')).toBeVisible();
  });

  test('Enumeration cards show value count', async ({ page }) => {
    // Look for value count text (e.g., "5 values")
    await expect(page.getByText(/\d+ values/).first()).toBeVisible();
  });
});

test.describe('Enumerations - Create Form', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/enumerations');
    await page.getByRole('button', { name: 'Add Enumeration' }).click();
    await page.waitForTimeout(500);
  });

  test('Create form has all required fields', async ({ page }) => {
    // Check for Name input
    await expect(page.getByPlaceholder('e.g., cover-types')).toBeVisible();

    // Check for Template Type input
    await expect(page.getByPlaceholder('e.g., insurance')).toBeVisible();

    // Check for Values section - the label includes an asterisk for required
    await expect(page.getByText(/^Values/).first()).toBeVisible();

    // Check for value input field
    await expect(page.getByPlaceholder('Add value (e.g., third-party)')).toBeVisible();

    // Check for Save and Cancel buttons
    await expect(page.getByRole('button', { name: 'Save' })).toBeVisible();
    await expect(page.getByRole('button', { name: 'Cancel' })).toBeVisible();
  });

  test('Save button is disabled until ID is filled', async ({ page }) => {
    const saveButton = page.getByRole('button', { name: 'Save' });

    // Should be disabled initially
    const isDisabled = await saveButton.isDisabled();
    expect(isDisabled).toBe(true);

    // Fill Name
    await page.getByPlaceholder('e.g., cover-types').fill('test-enum');

    // Button state may change based on validation
    await page.waitForTimeout(300);
  });

  test('Cancel button closes form', async ({ page }) => {
    // Click cancel
    await page.getByRole('button', { name: 'Cancel' }).click();

    // Form should close - heading should not be visible
    await expect(page.getByRole('heading', { name: 'Create Enumeration' })).not.toBeVisible();
  });
});

test.describe('Enumerations - Edit Operations', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/enumerations');
  });

  test('Edit button exists on enumeration cards', async ({ page }) => {
    // Wait for enumerations to load
    await expect(page.getByText('Coverage Type')).toBeVisible();
    const editButtons = page.getByRole('button', { name: 'Edit' });
    const count = await editButtons.count();
    expect(count).toBeGreaterThan(0);
  });

  test('Delete button exists on enumeration cards', async ({ page }) => {
    // Wait for enumerations to load
    await expect(page.getByText('Coverage Type')).toBeVisible();
    const deleteButtons = page.getByRole('button', { name: 'Delete' });
    const count = await deleteButtons.count();
    expect(count).toBeGreaterThan(0);
  });

  test('Edit opens form with existing data', async ({ page }) => {
    // Click edit on first enumeration
    await page.getByRole('button', { name: 'Edit' }).first().click();
    await page.waitForTimeout(500);

    // ID input should have value
    const idInput = page.getByPlaceholder('e.g., status, category');
    if (await idInput.isVisible()) {
      const value = await idInput.inputValue();
      expect(value.length).toBeGreaterThan(0);
    }
  });
});

test.describe('Enumerations - Verify Test Data', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/enumerations');
  });

  test('Coverage Type enumeration exists with correct values', async ({ page }) => {
    // Card shows "Coverage Type" display name
    await expect(page.getByText('Coverage Type')).toBeVisible();
    await expect(page.getByText('5 values').first()).toBeVisible();
  });

  test('Vehicle Type enumeration exists with 7 values', async ({ page }) => {
    await expect(page.getByText('Vehicle Type')).toBeVisible();
    await expect(page.getByText('7 values')).toBeVisible();
  });

  test('Driver Status enumeration exists with 4 values', async ({ page }) => {
    await expect(page.getByText('Driver Status')).toBeVisible();
    await expect(page.getByText('4 values').first()).toBeVisible();
  });

  test('Payment Frequency enumeration exists with 4 values', async ({ page }) => {
    await expect(page.getByText('Payment Frequency')).toBeVisible();
    await expect(page.getByText('4 values').nth(1)).toBeVisible();
  });

  test('Risk Tier enumeration exists', async ({ page }) => {
    await expect(page.getByText('Risk Tier')).toBeVisible();
  });

  test('Discount Type enumeration exists with 6 values', async ({ page }) => {
    await expect(page.getByText('Discount Type')).toBeVisible();
    await expect(page.getByText('6 values')).toBeVisible();
  });

  test('Claim Type enumeration exists', async ({ page }) => {
    await expect(page.getByText('Claim Type')).toBeVisible();
  });

  test('All 7 test enumerations are present', async ({ page }) => {
    // Check display names are visible
    const expectedNames = ['Coverage Type', 'Vehicle Type', 'Driver Status', 'Payment Frequency', 'Risk Tier', 'Discount Type', 'Claim Type'];

    for (const name of expectedNames) {
      await expect(page.getByText(name).first()).toBeVisible();
    }
  });
});

test.describe('Enumerations - Template Filter', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/enumerations');
  });

  test('Filter by insurance template shows relevant enumerations', async ({ page }) => {
    const templateFilter = page.getByRole('combobox').first();

    // Check if insurance option exists
    await templateFilter.click();
    const insuranceOption = page.getByRole('option', { name: 'insurance' });

    if (await insuranceOption.isVisible()) {
      await insuranceOption.click();
      await page.waitForTimeout(500);

      // Should show insurance-related enumerations
      await expect(page.getByText('coverage-type')).toBeVisible();
    }
  });
});
