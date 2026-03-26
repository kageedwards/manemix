// @vitest-environment happy-dom
import { describe, it, expect, vi, beforeEach } from 'vitest';
import fc from 'fast-check';
import { render } from '@testing-library/svelte';
import PlayerBar from '../PlayerBar.svelte';
import { playerState } from '$lib/stores/player';
import type { Track } from '$lib/types/index.js';
import type { PlayerState } from '$lib/stores/player';

// Mock Audio constructor
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

describe('PlayerBar', () => {
	beforeEach(() => {
		playerState.set({
			currentTrack: null, queue: [], queueIndex: 0,
			isPlaying: false, currentTime: 0, duration: 0, volume: 1
		});
	});

	/**
	 * Property 13: Player bar displays current track info and time
	 * Validates: Requirements 9.2, 9.5
	 */
	it('Property 13: displays current track info and time', () => {
		fc.assert(
			fc.property(
				arbTrack,
				fc.double({ min: 0, max: 3600, noNaN: true, noDefaultInfinity: true }),
				fc.double({ min: 1, max: 7200, noNaN: true, noDefaultInfinity: true }),
				(track, currentTime, duration) => {
					const state: PlayerState = {
						currentTrack: track, queue: [track], queueIndex: 0,
						isPlaying: true, currentTime, duration, volume: 1
					};
					playerState.set(state);

					const { container } = render(PlayerBar);
					const html = container.innerHTML;

					expect(html).toContain(track.title);
					expect(html).toContain(track.username);

					const timePattern = /\d+:\d{2}/;
					expect(html).toMatch(timePattern);
				}
			),
			{ numRuns: 50 }
		);
	});

	it('renders nothing when no track is playing', () => {
		const { container } = render(PlayerBar);
		expect(container.innerHTML.trim()).toBe('<!---->');
	});

	it('shows play/pause button', () => {
		const track: Track = {
			tid: 1, title: 'Test', uid: 1, username: 'artist',
			is_visible: true, is_hidden: false, date: '2024-01-01',
			timestamp: '', day: '', has_art: false
		};
		playerState.set({
			currentTrack: track, queue: [track], queueIndex: 0,
			isPlaying: true, currentTime: 30, duration: 180, volume: 1
		});
		const { container } = render(PlayerBar);
		const buttons = container.querySelectorAll('button');
		expect(buttons.length).toBeGreaterThan(0);
	});
});
