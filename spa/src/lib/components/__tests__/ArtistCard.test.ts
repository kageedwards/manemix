// @vitest-environment happy-dom
import { describe, it, expect } from 'vitest';
import fc from 'fast-check';
import { render } from '@testing-library/svelte';
import ArtistCard from '../ArtistCard.svelte';
import type { Artist } from '$lib/types/index.js';

const arbArtist: fc.Arbitrary<Artist> = fc.record({
	uid: fc.integer({ min: 1, max: 100000 }),
	username: fc.stringMatching(/^[A-Za-z0-9_]{1,50}$/),
	email_md5: fc.hexaString({ minLength: 32, maxLength: 32 }),
	about_html: fc.string(),
	has_about: fc.boolean()
});

describe('ArtistCard', () => {
	/**
	 * Property 12: Artist card renders username and avatar
	 * Validates: Requirements 8.2, 8.4
	 */
	it('Property 12: renders username and avatar', () => {
		fc.assert(
			fc.property(arbArtist, (artist) => {
				const { container } = render(ArtistCard, { props: { artist } });
				const html = container.innerHTML;

				expect(html).toContain(artist.username);

				const img = container.querySelector('img');
				expect(img).not.toBeNull();
				expect(img?.getAttribute('src')).toContain(artist.email_md5);

				const link = container.querySelector('a');
				expect(link?.getAttribute('href')).toBe(`/user/${artist.uid}`);
			}),
			{ numRuns: 50 }
		);
	});
});
