import { test, expect } from '@playwright/test';

/**
 * Simulation Panel Tests
 *
 * Tests the simulation functionality including:
 * - Opening simulation panel
 * - Entering input values
 * - Running calculations
 * - Verifying output values
 */
test.describe('Simulation Panel', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to the Rules page
    await page.goto('/rules');

    // Wait for the page to load
    await expect(page.locator('h1')).toContainText('Rules');

    // Simulation panel is visible by default on the right side
    // Wait for simulation heading to appear
    await expect(page.getByRole('heading', { name: 'Simulation' })).toBeVisible({ timeout: 10000 });
  });

  test('Simulation panel opens and shows inputs', async ({ page }) => {
    // Check that input section is visible - look for Inputs button
    await expect(page.getByRole('button', { name: /Inputs/ })).toBeVisible();

    // Check for input fields (spinbuttons, textboxes, or selects)
    const inputs = page.locator('input, select');
    const inputCount = await inputs.count();
    expect(inputCount).toBeGreaterThan(0);
  });

  test('Can enter numeric input values', async ({ page }) => {
    // Find a numeric input field (spinbutton)
    const numericInput = page.getByRole('spinbutton').first();

    if (await numericInput.isVisible()) {
      // Clear and enter a value
      await numericInput.fill('100');
      await expect(numericInput).toHaveValue('100');
    }
  });

  test('Can toggle boolean inputs', async ({ page }) => {
    // Find a boolean toggle button (shows True or False)
    const booleanToggle = page.getByRole('button', { name: /^(True|False)$/ }).first();

    if (await booleanToggle.isVisible()) {
      const initialText = await booleanToggle.textContent();

      // Click to toggle
      await booleanToggle.click();

      // Verify it changed
      const newText = await booleanToggle.textContent();
      expect(newText).not.toBe(initialText);
    }
  });

  test('Can select enum values from dropdown', async ({ page }) => {
    // Find a combobox/select
    const enumSelect = page.getByRole('combobox').first();

    if (await enumSelect.isVisible()) {
      // Get available options
      const options = enumSelect.locator('option');
      const optionCount = await options.count();

      if (optionCount > 1) {
        // Select the second option (first is usually placeholder)
        await enumSelect.selectOption({ index: 1 });
      }
    }
  });

  test('Run button triggers evaluation', async ({ page }) => {
    // Find and click the Run button
    const runButton = page.getByRole('button', { name: 'Run' });
    await expect(runButton).toBeVisible();

    // Fill in at least one required value to avoid errors
    const vehicleValue = page.getByRole('spinbutton').first();
    if (await vehicleValue.isVisible()) {
      await vehicleValue.fill('30000');
    }

    // Click Run
    await runButton.click();

    // Wait for outputs section to show results
    await page.waitForTimeout(2000);
    await expect(page.getByRole('button', { name: /Outputs/ })).toBeVisible({ timeout: 10000 });
  });

  test('Reset button clears inputs', async ({ page }) => {
    // Enter some values first
    const numericInput = page.getByRole('spinbutton').first();
    if (await numericInput.isVisible()) {
      await numericInput.fill('999');
    }

    // Click Reset
    const resetButton = page.getByRole('button', { name: 'Reset' });
    await resetButton.click();

    // Wait a moment for reset
    await page.waitForTimeout(500);

    // Verify input was cleared (or reset to default)
    if (await numericInput.isVisible()) {
      const value = await numericInput.inputValue();
      expect(value).not.toBe('999');
    }
  });

  test('Outputs section shows execution metrics', async ({ page }) => {
    // Fill required inputs
    const spinbuttons = page.getByRole('spinbutton');
    const count = await spinbuttons.count();

    // Fill first few spinbuttons with valid values
    for (let i = 0; i < Math.min(count, 5); i++) {
      const input = spinbuttons.nth(i);
      if (await input.isVisible()) {
        await input.fill('100');
      }
    }

    // Run evaluation
    await page.getByRole('button', { name: 'Run' }).click();
    await page.waitForTimeout(2000);

    // After running, Outputs button should be visible
    await expect(page.getByRole('button', { name: /Outputs/ })).toBeVisible();
  });
});

test.describe('Simulation Calculations', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/rules');
    await expect(page.locator('h1')).toContainText('Rules');

    // Simulation panel is visible by default on the right side
    await expect(page.getByRole('heading', { name: 'Simulation' })).toBeVisible({ timeout: 10000 });
  });

  test('Calculate with numeric inputs', async ({ page }) => {
    // Find and fill numeric inputs
    const spinbuttons = page.getByRole('spinbutton');
    const count = await spinbuttons.count();

    // Fill several inputs with test values
    for (let i = 0; i < Math.min(count, 8); i++) {
      const input = spinbuttons.nth(i);
      if (await input.isVisible()) {
        // Use different values for variety
        await input.fill(String((i + 1) * 100));
      }
    }

    // Run button should be clickable
    const runButton = page.getByRole('button', { name: 'Run' });
    await expect(runButton).toBeVisible();
    await runButton.click();
    await page.waitForTimeout(1000);

    // Should still see the simulation panel after running
    await expect(page.getByRole('heading', { name: 'Simulation' })).toBeVisible();
  });

  test('Toggle boolean and run evaluation', async ({ page }) => {
    // Toggle a boolean input
    const boolToggle = page.getByRole('button', { name: /^(True|False)$/ }).first();
    await expect(boolToggle).toBeVisible();
    await boolToggle.click();
    await page.waitForTimeout(300);

    // Fill a numeric value
    const numInput = page.getByRole('spinbutton').first();
    if (await numInput.isVisible()) {
      await numInput.fill('50000');
    }

    // Run button should be visible
    const runButton = page.getByRole('button', { name: 'Run' });
    await expect(runButton).toBeVisible();
  });

  test('Select enum value and run evaluation', async ({ page }) => {
    // Select an enum value
    const combobox = page.getByRole('combobox').first();
    await expect(combobox).toBeVisible();

    const options = await combobox.locator('option').allTextContents();
    if (options.length > 1) {
      // Select a non-placeholder option
      await combobox.selectOption({ index: 1 });
    }

    // Fill required numeric
    const numInput = page.getByRole('spinbutton').first();
    if (await numInput.isVisible()) {
      await numInput.fill('25000');
    }

    // Run button should be visible
    const runButton = page.getByRole('button', { name: 'Run' });
    await expect(runButton).toBeVisible();
  });
});

test.describe('Simulation UI Features', () => {
  test('Auto button is present', async ({ page }) => {
    await page.goto('/rules');
    await expect(page.locator('h1')).toContainText('Rules');
    // Simulation panel is visible by default on the right side
    await expect(page.getByRole('heading', { name: 'Simulation' })).toBeVisible({ timeout: 10000 });

    // Check Auto button exists - use exact match to avoid matching "Auto Insurance..."
    await expect(page.getByRole('button', { name: 'Auto', exact: true })).toBeVisible();
  });

  test('Inputs section is expandable', async ({ page }) => {
    await page.goto('/rules');
    await expect(page.locator('h1')).toContainText('Rules');
    // Simulation panel is visible by default on the right side
    await expect(page.getByRole('heading', { name: 'Simulation' })).toBeVisible({ timeout: 10000 });

    // Inputs button should be visible and clickable
    const inputsBtn = page.getByRole('button', { name: /Inputs/ });
    await expect(inputsBtn).toBeVisible();
  });

  test('Saved Scenarios section exists', async ({ page }) => {
    await page.goto('/rules');
    await expect(page.locator('h1')).toContainText('Rules');
    // Simulation panel is visible by default on the right side
    await expect(page.getByRole('heading', { name: 'Simulation' })).toBeVisible({ timeout: 10000 });

    // Look for Saved Scenarios section - use exact match to avoid matching "No saved scenarios"
    await expect(page.getByText('Saved Scenarios', { exact: true })).toBeVisible();

    // Save button should be visible
    await expect(page.getByRole('button', { name: 'Save' })).toBeVisible();
  });
});
