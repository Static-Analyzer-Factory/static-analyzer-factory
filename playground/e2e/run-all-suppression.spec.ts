/**
 * E2E test: verify "Run All" suppresses cross-checker noise.
 *
 * When clicking "Run All", the backend suppresses:
 * 1. null-deref findings when another checker flags the same allocation/deref
 * 2. generic-resource-leak findings when memory-leak covers the same source
 *
 * For each checker example, "Run All" should produce ONLY findings from
 * the intended checker (or closely related ones), not from unrelated checkers.
 */

import { test, expect } from '@playwright/test';

// Examples to test with "Run All" and expected allowed checker names
const RUN_ALL_TESTS = [
  {
    name: 'use-after-free',
    index: '5',
    allowed: ['use-after-free'],
    forbidden: ['null-deref', 'generic-resource-leak'],
  },
  {
    name: 'memory-leak',
    index: '7',
    allowed: ['memory-leak'],
    forbidden: ['null-deref', 'generic-resource-leak'],
  },
  {
    name: 'double-free',
    index: '8',
    allowed: ['double-free'],
    forbidden: ['null-deref', 'generic-resource-leak'],
  },
  {
    name: 'uninit-use',
    index: '12',
    allowed: ['uninit-use'],
    forbidden: ['null-deref'],
  },
  {
    name: 'generic-resource-leak',
    index: '14',
    // memory-leak fires because specs are identical; generic-resource-leak
    // is suppressed in favor of memory-leak. Both are correct.
    allowed: ['memory-leak'],
    forbidden: ['null-deref', 'generic-resource-leak'],
  },
];

test.describe('"Run All" cross-checker suppression', () => {
  for (const { name, index, allowed, forbidden } of RUN_ALL_TESTS) {
    test(`${name}: Run All shows only intended findings`, async ({ page }) => {
      // 1. Navigate and select example
      await page.goto('/');
      const exampleSelect = page.locator('select').first();
      await exampleSelect.selectOption(index);

      // 2. Click Analyze
      await page.locator('button.btn-primary').click();

      // 3. Wait for analysis
      await page.locator('.status-dot.ready').waitFor({ timeout: 90_000 });

      // 4. Switch to Query tab → Checks sub-tab
      await page.locator('button.header-tab', { hasText: 'Query' }).click();
      await page.locator('button.query-tab', { hasText: 'Checks' }).click();

      // 5. Click "Run All"
      await page.locator('button.query-btn', { hasText: 'Run All' }).click();

      // 6. Wait for findings
      await page.waitForTimeout(3000);

      // 7. Collect all finding checker names from the finding cards
      const findingCards = page.locator('.query-finding-card');
      const count = await findingCards.count();
      console.log(`${name}: Run All found ${count} finding(s)`);

      // We should have at least 1 finding (from the intended checker)
      expect(count, `${name}: Run All should find >= 1 bug`).toBeGreaterThanOrEqual(1);

      // 8. Extract checker names from finding cards
      // Finding cards show the checker name in a badge/label
      const checkerNames: string[] = [];
      for (let i = 0; i < count; i++) {
        const card = findingCards.nth(i);
        const text = await card.textContent();
        checkerNames.push(text ?? '');
      }

      // Log findings BEFORE assertions for debugging
      console.log(`${name}: Findings: ${checkerNames.map(t => t.substring(0, 60)).join(' | ')}`);

      // 9. Verify no forbidden checkers appear
      for (const f of forbidden) {
        const hasForbidden = checkerNames.some(t => t.includes(f));
        expect(hasForbidden, `${name}: should NOT have ${f} findings, but got: ${checkerNames.join(' | ')}`).toBe(false);
      }

      // 10. Verify at least one allowed checker appears
      const hasAllowed = checkerNames.some(t => allowed.some(a => t.includes(a)));
      expect(hasAllowed, `${name}: should have findings from ${allowed.join('/')}`).toBe(true);
    });
  }
});
