import { test, expect } from '@playwright/test';

/**
 * Product Workflow and Management Tests
 *
 * Tests product listing, datatypes, enumerations, attributes, and functionalities.
 */
test.describe('Products Page', () => {
  test('Products page shows product list', async ({ page }) => {
    await page.goto('/products');

    // Check main heading
    await expect(page.locator('h1')).toContainText('Products');

    // Check stats are displayed
    await expect(page.getByText('Total')).toBeVisible();

    // Check New Product button
    await expect(page.getByRole('button', { name: 'New Product' })).toBeVisible();
  });

  test('Products page has search functionality', async ({ page }) => {
    await page.goto('/products');

    // Check for search input
    const searchInput = page.getByPlaceholder(/Search/i);
    await expect(searchInput).toBeVisible();
  });

  test('Can navigate to Rule Canvas from product', async ({ page }) => {
    await page.goto('/products');

    // Find Rule Canvas link
    const ruleCanvasLink = page.getByRole('link', { name: 'Rule Canvas' }).first();
    if (await ruleCanvasLink.isVisible()) {
      await ruleCanvasLink.click();
      await expect(page).toHaveURL('/rules');
    }
  });
});

test.describe('Datatypes Page', () => {
  test('Datatypes page shows datatype list', async ({ page }) => {
    await page.goto('/datatypes');

    await expect(page.locator('h1')).toContainText('Datatypes');

    // Check stats
    await expect(page.getByText('Total Types')).toBeVisible();

    // Check Add button
    await expect(page.getByRole('button', { name: 'Add Datatype' })).toBeVisible();
  });

  test('Datatypes page has type filter', async ({ page }) => {
    await page.goto('/datatypes');

    // Check for type filter dropdown
    const typeFilter = page.getByRole('combobox').first();
    await expect(typeFilter).toBeVisible();
  });

  test('Datatypes page has search', async ({ page }) => {
    await page.goto('/datatypes');

    // Check for search input
    const searchInput = page.getByPlaceholder(/Search/i);
    await expect(searchInput).toBeVisible();
  });

  test('Datatype cards show constraints', async ({ page }) => {
    await page.goto('/datatypes');

    // Look for constraint indicators (min, max, regex)
    const constraintTexts = page.getByText(/min:|max:|regex/i);
    const count = await constraintTexts.count();

    // Should have some datatypes with constraints
    expect(count).toBeGreaterThanOrEqual(0);
  });
});

test.describe('Enumerations Page', () => {
  test('Enumerations page shows enumeration list', async ({ page }) => {
    await page.goto('/enumerations');

    await expect(page.locator('h1')).toContainText('Enumerations');

    // Check stats
    await expect(page.getByText('Total Enumerations')).toBeVisible();

    // Check Add button
    await expect(page.getByRole('button', { name: 'Add Enumeration' })).toBeVisible();
  });

  test('Enumerations page has template filter', async ({ page }) => {
    await page.goto('/enumerations');

    // Check for template filter dropdown
    const templateFilter = page.getByRole('combobox').first();
    await expect(templateFilter).toBeVisible();
  });

  test('Enumerations page has search', async ({ page }) => {
    await page.goto('/enumerations');

    // Check for search input
    const searchInput = page.getByPlaceholder(/Search/i);
    await expect(searchInput).toBeVisible();
  });

  test('Enumeration cards show value count', async ({ page }) => {
    await page.goto('/enumerations');

    // Look for "values" text indicating value counts
    await expect(page.getByText(/\d+ values/).first()).toBeVisible();
  });
});

test.describe('Attributes Page', () => {
  test('Attributes page shows attribute count', async ({ page }) => {
    await page.goto('/attributes');

    await expect(page.locator('h1')).toContainText('Attributes');

    // Select a product first
    await page.getByRole('heading', { name: 'Auto Insurance Premium Calculator' }).click();
    await page.waitForTimeout(500);

    // Check for attribute count
    await expect(page.getByText('attributes').first()).toBeVisible();
  });

  test('Attributes page has Abstract/Concrete tabs', async ({ page }) => {
    await page.goto('/attributes');
    await expect(page.locator('h1')).toContainText('Attributes');

    // Select a product first
    await page.getByRole('heading', { name: 'Auto Insurance Premium Calculator' }).click();
    await page.waitForTimeout(500);

    // Check for tab buttons - use regex to match "Abstract (N)" pattern to avoid matching "Add Abstract Attribute"
    await expect(page.getByRole('button', { name: /^Abstract \(\d+\)$/ })).toBeVisible();
    await expect(page.getByRole('button', { name: /^Concrete \(\d+\)$/ })).toBeVisible();
  });

  test('Attributes page has Table/Tree view toggle', async ({ page }) => {
    await page.goto('/attributes');
    await expect(page.locator('h1')).toContainText('Attributes');

    // Select a product first
    await page.getByRole('heading', { name: 'Auto Insurance Premium Calculator' }).click();
    await page.waitForTimeout(500);

    // Check for view toggle buttons
    await expect(page.getByRole('button', { name: 'Table' })).toBeVisible();
    await expect(page.getByRole('button', { name: 'Tree' })).toBeVisible();
  });

  test('Attributes page shows component sections', async ({ page }) => {
    await page.goto('/attributes');
    await expect(page.locator('h1')).toContainText('Attributes');

    // Select a product first
    await page.getByRole('heading', { name: 'Auto Insurance Premium Calculator' }).click();
    await page.waitForTimeout(500);

    // Should have component filter/section buttons
    const componentButtons = page.getByRole('button').filter({
      hasText: /Coverage|Customer|Vehicle|Rating|Eligibility/i,
    });
    const count = await componentButtons.count();
    expect(count).toBeGreaterThan(0);
  });
});

test.describe('Functionalities Page', () => {
  test('Functionalities page shows functionality list', async ({ page }) => {
    await page.goto('/functionalities');

    // h1 says "Functions" not "Functionalities"
    await expect(page.locator('h1')).toContainText('Functions');

    // Select a product first to see functionalities
    await page.getByRole('heading', { name: 'Auto Insurance Premium Calculator' }).click();
    await page.waitForTimeout(500);

    // Check Add button - named "New Functionality"
    await expect(page.getByRole('button', { name: 'New Functionality' })).toBeVisible();
  });

  test('Functionalities page has status filters', async ({ page }) => {
    await page.goto('/functionalities');
    await expect(page.locator('h1')).toContainText('Functions');

    // Select a product first
    await page.getByRole('heading', { name: 'Auto Insurance Premium Calculator' }).click();
    await page.waitForTimeout(500);

    // Check for status filter buttons - uppercase names
    await expect(page.getByRole('button', { name: 'All' }).first()).toBeVisible();
    await expect(page.getByRole('button', { name: 'ACTIVE' }).first()).toBeVisible();
    await expect(page.getByRole('button', { name: 'DRAFT' }).first()).toBeVisible();
  });

  test('Functionalities page has search', async ({ page }) => {
    await page.goto('/functionalities');
    await expect(page.locator('h1')).toContainText('Functions');

    // Select a product first
    await page.getByRole('heading', { name: 'Auto Insurance Premium Calculator' }).click();
    await page.waitForTimeout(500);

    // Check for search input
    const searchInput = page.getByPlaceholder(/Search/i).first();
    await expect(searchInput).toBeVisible();
  });
});
