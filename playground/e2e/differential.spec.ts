/**
 * Differential E2E test: verify that both query checkers and Python
 * analyzers detect bugs in the playground for all 9 checker examples.
 *
 * For each checker example:
 * 1. Select the example, compile & analyze
 * 2. Run the query checker → expect >= 1 finding
 * 3. Run the matching Python analyzer → expect >= 1 finding
 */

import { test, expect } from '@playwright/test';

// Map example → { index in examples array, checker name, analyzer template key }
const CHECKER_EXAMPLES = [
  { name: 'use-after-free', index: '5', checker: 'use_after_free', template: 'detect_uaf' },
  { name: 'memory-leak', index: '7', checker: 'memory_leak', template: 'detect_memory_leak' },
  { name: 'double-free', index: '8', checker: 'double_free', template: 'detect_double_free' },
  { name: 'null-deref', index: '9', checker: 'null_deref', template: 'detect_null_deref' },
  { name: 'file-descriptor-leak', index: '10', checker: 'file_descriptor_leak', template: 'detect_file_leak' },
  { name: 'lock-not-released', index: '11', checker: 'lock_not_released', template: 'detect_lock_leak' },
  { name: 'uninit-use', index: '12', checker: 'uninit_use', template: 'detect_uninit_use' },
  { name: 'stack-escape', index: '13', checker: 'stack_escape', template: 'detect_stack_escape' },
  { name: 'generic-resource-leak', index: '14', checker: 'generic_resource_leak', template: 'detect_memory_leak' },
];

test.describe('Differential testing: Query checkers vs Python analyzers', () => {
  for (const { name, index, checker, template } of CHECKER_EXAMPLES) {
    test(`${name}: query checker and python analyzer both find bugs`, async ({ page }) => {
      // 1. Navigate to playground
      await page.goto('/');

      // 2. Select the example from the dropdown (value is numeric index)
      const exampleSelect = page.locator('select').first();
      await exampleSelect.selectOption(index);

      // 3. Click Analyze button
      await page.locator('button.btn-primary').click();

      // 4. Wait for analysis to complete (status dot turns "ready")
      await page.locator('.status-dot.ready').waitFor({ timeout: 90_000 });

      // --- Query checker path ---

      // 5. Switch to Query tab
      await page.locator('button.header-tab', { hasText: 'Query' }).click();

      // 6. Ensure Checks sub-tab is active
      await page.locator('button.query-tab', { hasText: 'Checks' }).click();

      // 7. Select the specific checker from the dropdown
      const checkerSelect = page.locator('.query-controls select');
      await checkerSelect.selectOption(checker);

      // 8. Click Run Check
      await page.locator('button.query-btn', { hasText: 'Run Check' }).click();

      // 9. Wait for findings to appear (or "no findings" state)
      await page.waitForTimeout(3000);

      // 10. Count query findings
      const queryFindings = page.locator('.query-finding-card');
      const queryCount = await queryFindings.count();
      console.log(`${name}: Query checker found ${queryCount} finding(s)`);

      // Assert query checker found at least 1 bug
      expect(queryCount, `Query checker ${checker} should find >= 1 bug`).toBeGreaterThanOrEqual(1);

      // --- Python analyzer path ---

      // 11. Switch to Analyzer tab
      await page.locator('button.header-tab', { hasText: 'Analyzer' }).click();

      // 12. Select the matching template
      const templateSelect = page.locator('.analyzer-toolbar select');
      await templateSelect.selectOption(template);

      // 13. Wait for Pyodide to be ready (if not already)
      await page.locator('.pyodide-dot.ready').waitFor({ timeout: 60_000 });

      // 14. Click Run
      await page.locator('button.btn-run').click();

      // 15. Wait for "Running…" to appear, then disappear (script complete)
      // Note: hasText: 'Run' matches 'Running…' too, so use exact regex
      await page.waitForTimeout(500); // Let React update to "Running…"
      await page.locator('button.btn-run', { hasText: /^Run$/ }).waitFor({ timeout: 90_000 });
      // Extra wait for findings to be committed to state and rendered
      await page.waitForTimeout(3000);

      // 16. Switch to Findings output tab
      await page.locator('button.analyzer-output-tab', { hasText: 'Findings' }).click();

      // 17. Count Python analyzer findings
      const pyFindings = page.locator('.finding-card');
      const pyCount = await pyFindings.count();
      console.log(`${name}: Python analyzer found ${pyCount} finding(s)`);

      // Assert Python analyzer found at least 1 bug
      expect(pyCount, `Python analyzer for ${name} should find >= 1 bug`).toBeGreaterThanOrEqual(1);
    });
  }
});
