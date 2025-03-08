import { test, expect } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';

test.describe('Accessibility Tests', () => {
  test('should pass accessibility scan on main pages', async ({ page }) => {
    // Test main pages
    const pagesToTest = [
      '/',
      '/dashboard',
      '/federations',
      '/governance',
      '/resources'
    ];

    for (const path of pagesToTest) {
      await page.goto(`${process.env.TEST_API_URL}${path}`);
      await page.waitForLoadState('networkidle');

      const accessibilityScanResults = await new AxeBuilder({ page })
        .withTags(['wcag2a', 'wcag2aa', 'wcag21aa'])
        .analyze();

      expect(accessibilityScanResults.violations).toEqual([]);
    }
  });

  test('should pass accessibility scan on critical flows', async ({ page }) => {
    // Test federation creation flow
    await page.goto(`${process.env.TEST_API_URL}/federations/create`);
    await page.waitForLoadState('networkidle');
    
    const createFederationResults = await new AxeBuilder({ page })
      .withTags(['wcag2a', 'wcag2aa', 'wcag21aa'])
      .analyze();

    expect(createFederationResults.violations).toEqual([]);

    // Test governance voting flow
    await page.goto(`${process.env.TEST_API_URL}/governance/proposals/new`);
    await page.waitForLoadState('networkidle');
    
    const createProposalResults = await new AxeBuilder({ page })
      .withTags(['wcag2a', 'wcag2aa', 'wcag21aa'])
      .analyze();

    expect(createProposalResults.violations).toEqual([]);
  });
});