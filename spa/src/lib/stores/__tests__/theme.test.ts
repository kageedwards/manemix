// @vitest-environment happy-dom
import { describe, it, expect, beforeEach } from 'vitest';
import fc from 'fast-check';
import { get } from 'svelte/store';
import { theme, toggleTheme } from '../theme';

describe('Theme Store Property Tests', () => {
	beforeEach(() => {
		localStorage.clear();
	});

	/**
	 * Property 17: Theme toggle switches between dark and light
	 * For any current theme state (dark or light), toggling the theme
	 * should switch to the opposite theme and persist the preference.
	 *
	 * **Validates: Requirements 2.7, 15.5**
	 */
	it('Property 17: theme toggle switches between dark and light', () => {
		fc.assert(
			fc.property(
				fc.constantFrom('dark' as const, 'light' as const),
				(initialTheme) => {
					// Set the store to the initial theme
					theme.set(initialTheme);

					// Verify initial state
					expect(get(theme)).toBe(initialTheme);

					// Toggle
					toggleTheme();
					const stateAfter = get(theme);

					// Should be the opposite
					const expected = initialTheme === 'dark' ? 'light' : 'dark';
					expect(stateAfter).toBe(expected);

					// Verify persistence: localStorage has the new theme
					expect(localStorage.getItem('theme')).toBe(expected);

					// Verify data-theme attribute was updated
					const expectedDaisyTheme =
						expected === 'dark' ? 'manemix_dark' : 'manemix_light';
					expect(document.documentElement.getAttribute('data-theme')).toBe(
						expectedDaisyTheme
					);
				}
			),
			{ numRuns: 100 }
		);
	});
});
