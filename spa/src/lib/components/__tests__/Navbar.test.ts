// @vitest-environment happy-dom
import { describe, it, expect, vi, beforeEach } from 'vitest';
import fc from 'fast-check';
import { render } from '@testing-library/svelte';
import Navbar from '../Navbar.svelte';
import { auth } from '$lib/stores/auth';
import type { MeResponse } from '$lib/types/index.js';

// Mock Audio for player store (imported transitively)
const mockAudio = {
	src: '', currentTime: 0, duration: 0, volume: 1,
	play: vi.fn().mockResolvedValue(undefined), pause: vi.fn(),
	addEventListener: vi.fn(), canPlayType: vi.fn().mockReturnValue('')
};
vi.stubGlobal('Audio', vi.fn(() => mockAudio));

describe('Navbar', () => {
	beforeEach(() => {
		auth.set({ logged_in: false });
	});

	/**
	 * Property 19: Navbar renders auth-conditional content
	 * Validates: Requirements 15.2, 15.3
	 */
	it('Property 19: renders auth-conditional content', () => {
		fc.assert(
			fc.property(
				fc.record({
					logged_in: fc.boolean(),
					uid: fc.integer({ min: 1, max: 100000 }),
					username: fc.stringMatching(/^[A-Za-z0-9_]{1,50}$/)
				}),
				(data) => {
					const meResponse: MeResponse = data.logged_in
						? { logged_in: true, uid: data.uid, username: data.username }
						: { logged_in: false };

					auth.set(meResponse);
					const { container } = render(Navbar);
					const html = container.innerHTML;

					if (data.logged_in) {
						expect(html).toContain(data.username);
						expect(html).toContain('Logout');
					} else {
						expect(html).toContain('Login');
					}
				}
			),
			{ numRuns: 50 }
		);
	});

	it('shows site title linking to home', () => {
		const { container } = render(Navbar);
		const titleLink = container.querySelector('a[href="/"]');
		expect(titleLink).not.toBeNull();
		expect(titleLink?.textContent).toContain('Manehattan Mix');
	});

	it('shows nav links for Home, Latest, Artists', () => {
		const { container } = render(Navbar);
		const links = Array.from(container.querySelectorAll('a'));
		const hrefs = links.map((a) => a.getAttribute('href'));
		expect(hrefs).toContain('/');
		expect(hrefs).toContain('/tracks/latest');
		expect(hrefs).toContain('/artists');
	});
});
