import { describe, it, expect, vi, beforeEach } from 'vitest';
import fc from 'fast-check';
import { get } from 'svelte/store';
import type { Track } from '$lib/types/index.js';
import { play, seek, next, getAudioUrl, playerState } from '../player';

// --- Mock browser APIs ---

const mockAudio = {
	src: '',
	currentTime: 0,
	duration: 0,
	volume: 1,
	play: vi.fn().mockResolvedValue(undefined),
	pause: vi.fn(),
	addEventListener: vi.fn(),
	canPlayType: vi.fn().mockReturnValue('')
};

vi.stubGlobal('Audio', vi.fn(() => mockAudio));

// --- Arbitrary generators ---

const arbTrack: fc.Arbitrary<Track> = fc.record({
	tid: fc.integer({ min: 1, max: 100000 }),
	title: fc.string({ minLength: 1, maxLength: 100 }),
	uid: fc.integer({ min: 1, max: 100000 }),
	username: fc.string({ minLength: 1, maxLength: 50 }),
	is_visible: fc.boolean(),
	is_hidden: fc.boolean(),
	date: fc.string(),
	timestamp: fc.string(),
	day: fc.string(),
	has_art: fc.boolean()
});

// Generate an array of tracks with unique tids
const arbUniqueTracks = (minLength: number, maxLength: number) =>
	fc
		.uniqueArray(fc.integer({ min: 1, max: 100000 }), {
			minLength,
			maxLength
		})
		.chain((tids) =>
			fc.tuple(...tids.map((tid) => arbTrack.map((t) => ({ ...t, tid })))).map((arr) => arr)
		);

describe('Player Store Property Tests', () => {
	beforeEach(() => {
		mockAudio.src = '';
		mockAudio.currentTime = 0;
		mockAudio.duration = 0;
		mockAudio.volume = 1;
		mockAudio.play.mockResolvedValue(undefined);
		mockAudio.pause.mockClear();
		mockAudio.addEventListener.mockClear();
		mockAudio.canPlayType.mockReturnValue('');
	});

	/**
	 * Property 3: Player play action sets current track and queue
	 * For any track list and any valid index within that list, calling
	 * play(tracks[index], tracks) should set currentTrack to tracks[index],
	 * set the queue to the full track list, set queueIndex to the given index,
	 * and set isPlaying to true.
	 *
	 * **Validates: Requirements 3.4, 6.4, 9.8, 14.3**
	 */
	it('Property 3: play action sets current track and queue', () => {
		fc.assert(
			fc.property(
				arbUniqueTracks(1, 20).chain((tracks) =>
					fc.integer({ min: 0, max: tracks.length - 1 }).map((index) => ({ tracks, index }))
				),
				({ tracks, index }) => {
					play(tracks[index], tracks);
					const state = get(playerState);

					expect(state.currentTrack).toEqual(tracks[index]);
					expect(state.queue).toEqual(tracks);
					expect(state.queueIndex).toBe(index);
					expect(state.isPlaying).toBe(true);
				}
			),
			{ numRuns: 100 }
		);
	});

	/**
	 * Property 14: Player seek updates current time
	 * For any seek position t where 0 <= t <= duration, calling seek(t)
	 * should set currentTime to t.
	 *
	 * **Validates: Requirements 9.4**
	 */
	it('Property 14: player seek updates current time', () => {
		fc.assert(
			fc.property(
				fc.double({ min: 0, max: 3600, noNaN: true, noDefaultInfinity: true }),
				(seekTime) => {
					seek(seekTime);
					const state = get(playerState);
					expect(state.currentTime).toBe(seekTime);
				}
			),
			{ numRuns: 100 }
		);
	});

	/**
	 * Property 15: Player auto-advances to next queued track
	 * For any queue with queueIndex < queue.length - 1, calling next()
	 * should set currentTrack to queue[queueIndex + 1] and increment queueIndex.
	 *
	 * **Validates: Requirements 9.10**
	 */
	it('Property 15: player auto-advances to next queued track', () => {
		fc.assert(
			fc.property(
				arbUniqueTracks(2, 20).chain((tracks) =>
					fc.integer({ min: 0, max: tracks.length - 2 }).map((index) => ({ tracks, index }))
				),
				({ tracks, index }) => {
					// Set up state by playing the track at the given index
					play(tracks[index], tracks);

					// Advance to next
					next();
					const state = get(playerState);

					expect(state.currentTrack).toEqual(tracks[index + 1]);
					expect(state.queueIndex).toBe(index + 1);
				}
			),
			{ numRuns: 100 }
		);
	});

	/**
	 * Property 16: Player selects Opus with MP3 fallback
	 * For any track ID and boolean canPlayOpus:
	 * - If canPlayOpus is true → URL should be /track/{tid}/opus?stream=1
	 * - If canPlayOpus is false → URL should be /track/{tid}/mp3?stream=1
	 *
	 * **Validates: Requirements 9.9**
	 */
	it('Property 16: player selects Opus with MP3 fallback', () => {
		fc.assert(
			fc.property(
				fc.integer({ min: 1, max: 100000 }),
				fc.boolean(),
				(tid, canPlayOpus) => {
					const url = getAudioUrl(tid, canPlayOpus);

					if (canPlayOpus) {
						expect(url).toBe(`/api/v1/track/${tid}/opus?stream=1`);
					} else {
						expect(url).toBe(`/api/v1/track/${tid}/mp3?stream=1`);
					}
				}
			),
			{ numRuns: 100 }
		);
	});
});
