// @vitest-environment happy-dom
import { describe, it, expect, vi } from 'vitest';
import fc from 'fast-check';
import { render } from '@testing-library/svelte';
import TrackCard from '../TrackCard.svelte';
import type { Track } from '$lib/types/index.js';

// Mock Audio constructor for player store
const mockAudio = {
	src: '', currentTime: 0, duration: 0, volume: 1,
	play: vi.fn().mockResolvedValue(undefined), pause: vi.fn(),
	addEventListener: vi.fn(), canPlayType: vi.fn().mockReturnValue('')
};
vi.stubGlobal('Audio', vi.fn(() => mockAudio));

const arbTrack: fc.Arbitrary<Track> = fc.record({
	tid: fc.integer({ min: 1, max: 100000 }),
	title: fc.stringMatching(/^[A-Za-z0-9 ]{1,80}$/),
	uid: fc.integer({ min: 1, max: 100000 }),
	username: fc.stringMatching(/^[A-Za-z0-9_]{1,50}$/),
	is_visible: fc.boolean(),
	is_hidden: fc.boolean(),
	date: fc.stringMatching(/^[0-9\-]{1,20}$/),
	timestamp: fc.string(),
	day: fc.string(),
	has_art: fc.boolean()
});

describe('TrackCard', () => {
	/**
	 * Property 1: TrackCard renders all required fields
	 * Validates: Requirements 3.2, 3.3
	 */
	it('Property 1: renders all required fields', () => {
		fc.assert(
			fc.property(arbTrack, (track) => {
				const { container } = render(TrackCard, { props: { track } });
				const html = container.innerHTML;

				expect(html).toContain(track.title);
				expect(html).toContain(track.username);
				expect(html).toContain(track.date);

				if (track.has_art) {
					const img = container.querySelector('img');
					expect(img).not.toBeNull();
					expect(img?.getAttribute('src')).toContain(`/track/${track.tid}/art/thumb`);
				}
			}),
			{ numRuns: 50 }
		);
	});

	/**
	 * Property 2: TrackCard links point to correct routes
	 * Validates: Requirements 3.5, 8.4
	 */
	it('Property 2: links point to correct routes', () => {
		fc.assert(
			fc.property(arbTrack, (track) => {
				const { container } = render(TrackCard, { props: { track } });

				// Verify a link to the track detail page exists
				const trackLinks = container.querySelectorAll(`a[href="/track/${track.tid}"]`);
				expect(trackLinks.length).toBeGreaterThan(0);

				// Verify a link to the artist profile exists
				const artistLink = container.querySelector(`a[href="/user/${track.uid}"]`);
				expect(artistLink).not.toBeNull();
			}),
			{ numRuns: 50 }
		);
	});

	it('renders without art when has_art is false', () => {
		const track: Track = {
			tid: 1, title: 'Test Track', uid: 2, username: 'artist',
			is_visible: true, is_hidden: false, date: '2024-01-01',
			timestamp: '', day: '', has_art: false
		};
		const { container } = render(TrackCard, { props: { track } });
		expect(container.querySelector('img')).toBeNull();
	});

	it('renders with art when has_art is true', () => {
		const track: Track = {
			tid: 42, title: 'Art Track', uid: 3, username: 'painter',
			is_visible: true, is_hidden: false, date: '2024-06-15',
			timestamp: '', day: '', has_art: true
		};
		const { container } = render(TrackCard, { props: { track } });
		const img = container.querySelector('img');
		expect(img).not.toBeNull();
		expect(img?.getAttribute('src')).toBe('/api/v1/track/42/art/thumb');
	});
});
