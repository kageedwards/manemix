import { writable, get } from 'svelte/store';
import type { Track, PlaybackContext } from '$lib/types/index.js';
import { getNextTracks } from '$lib/api/client.js';

export interface PlayerState {
	currentTrack: Track | null;
	queue: Track[];
	queueIndex: number;
	isPlaying: boolean;
	currentTime: number;
	duration: number;
	volume: number; // 0–1
	/** Describes the list the user started playing from, so we can fetch more. */
	playbackContext: PlaybackContext | null;
}

const DEFAULT_STATE: PlayerState = {
	currentTrack: null,
	queue: [],
	queueIndex: 0,
	isPlaying: false,
	currentTime: 0,
	duration: 0,
	volume: 1,
	playbackContext: null
};

export const playerState = writable<PlayerState>(DEFAULT_STATE);

// --- Audio element (lazy, SSR-safe) ---

let audio: HTMLAudioElement | null = null;

/** Get the Audio element (for visualizer connection). */
export function getAudioElement(): HTMLAudioElement | null {
	return audio;
}

function getAudio(): HTMLAudioElement | null {
	if (typeof window === 'undefined') return null;
	if (!audio) {
		audio = new Audio();
		audio.addEventListener('timeupdate', () => {
			playerState.update((s) => ({ ...s, currentTime: audio!.currentTime }));
		});
		audio.addEventListener('loadedmetadata', () => {
			playerState.update((s) => ({ ...s, duration: audio!.duration }));
		});
		audio.addEventListener('ended', () => {
			next();
		});
	}
	return audio;
}

// --- Helpers ---

/**
 * Check whether the browser can play Opus in an Ogg container.
 * Exported for testability.
 */
export function canPlayOpus(): boolean {
	if (typeof window === 'undefined') return false;
	const a = getAudio();
	if (!a) return false;
	const result = a.canPlayType('audio/ogg; codecs=opus');
	return result === 'probably' || result === 'maybe';
}

/**
 * Build the streaming URL for a track.
 * Exported as a pure helper for easy testing.
 */
export function getAudioUrl(tid: number, opusSupported: boolean): string {
	if (opusSupported) {
		return `/api/v1/track/${tid}/opus?stream=1`;
	}
	return `/api/v1/track/${tid}/mp3?stream=1`;
}

// Guard against concurrent fetches
let fetchingMore = false;

/**
 * If we're on the last track in the local queue and have a playback context,
 * eagerly fetch the next chunk so it's ready by the time the track ends.
 */
function prefetchIfNeeded(): void {
	const state = get(playerState);
	if (
		fetchingMore ||
		!state.playbackContext ||
		!state.currentTrack ||
		state.queueIndex < state.queue.length - 1
	) {
		return;
	}

	fetchingMore = true;
	getNextTracks(state.currentTrack.tid, state.playbackContext)
		.then((more) => {
			if (more.length > 0) {
				// Append the chunk to the queue so next() can use it immediately.
				playerState.update((s) => ({ ...s, queue: [...s.queue, ...more] }));
			}
		})
		.catch(() => {
			// Silently ignore — next() will just stop at the end.
		})
		.finally(() => {
			fetchingMore = false;
		});
}

// --- Actions ---

/**
 * Play a track. Optionally provide a queue and a playback context.
 * The context tells the player what list this track belongs to so it can
 * fetch more when the local queue runs out.
 */
export function play(track: Track, queue?: Track[], context?: PlaybackContext): void {
	const el = getAudio();

	const resolvedQueue = queue ?? [track];
	const queueIndex = queue ? queue.findIndex((t) => t.tid === track.tid) : 0;
	const idx = queueIndex === -1 ? 0 : queueIndex;

	playerState.set({
		currentTrack: track,
		queue: resolvedQueue,
		queueIndex: idx,
		isPlaying: true,
		currentTime: 0,
		duration: 0,
		volume: get(playerState).volume,
		playbackContext: context ?? null
	});

	if (el) {
		const url = getAudioUrl(track.tid, canPlayOpus());
		el.src = url;
		el.play().catch(() => {
			playerState.update((s) => ({ ...s, isPlaying: false }));
		});
	}

	// Eagerly fetch more if we're already on the last track in the queue.
	prefetchIfNeeded();
}

/** Pause playback. */
export function pause(): void {
	const el = getAudio();
	if (el) {
		el.pause();
	}
	playerState.update((s) => ({ ...s, isPlaying: false }));
}

/** Resume playback. */
export function resume(): void {
	const el = getAudio();
	if (el) {
		el.play().catch(() => {
			playerState.update((s) => ({ ...s, isPlaying: false }));
		});
	}
	playerState.update((s) => ({ ...s, isPlaying: true }));
}

/**
 * Advance to the next track in the queue.
 * Normally the prefetch has already appended more tracks, so this is instant.
 * If the prefetch hasn't landed yet (slow network), we await it as a fallback.
 */
export async function next(): Promise<void> {
	const state = get(playerState);

	// Tracks ahead in the local queue — just advance.
	if (state.queueIndex < state.queue.length - 1) {
		const nextIndex = state.queueIndex + 1;
		const nextTrack = state.queue[nextIndex];
		playInternal(nextTrack, state.queue, nextIndex, state.playbackContext);
		return;
	}

	// Prefetch didn't land in time — try one synchronous fetch as fallback.
	if (state.playbackContext && state.currentTrack && !fetchingMore) {
		fetchingMore = true;
		try {
			const more = await getNextTracks(state.currentTrack.tid, state.playbackContext);
			if (more.length > 0) {
				const newQueue = [...state.queue, ...more];
				const nextIndex = state.queueIndex + 1;
				playInternal(more[0], newQueue, nextIndex, state.playbackContext);
				return;
			}
		} catch {
			// Network error — stop gracefully.
		} finally {
			fetchingMore = false;
		}
	}

	// Nothing more to play.
	playerState.update((s) => ({ ...s, isPlaying: false }));
}

/** Go to the previous track, or restart current track if >5s in or at start of queue. */
export function prev(): void {
	const state = get(playerState);
	if (state.currentTime >= 5 || state.queueIndex <= 0) {
		seek(0);
		return;
	}
	const prevIndex = state.queueIndex - 1;
	const prevTrack = state.queue[prevIndex];
	playInternal(prevTrack, state.queue, prevIndex, state.playbackContext);
}

/** Seek to a specific time (in seconds). */
export function seek(time: number): void {
	const el = getAudio();
	if (el) {
		el.currentTime = time;
	}
	playerState.update((s) => ({ ...s, currentTime: time }));
}

/** Set the volume (0–1). */
export function setVolume(v: number): void {
	const clamped = Math.max(0, Math.min(1, v));
	const el = getAudio();
	if (el) {
		el.volume = clamped;
	}
	playerState.update((s) => ({ ...s, volume: clamped }));
}

// --- Internal ---

/** Start playing a track at a known queue position without resetting context. */
function playInternal(
	track: Track,
	queue: Track[],
	index: number,
	context: PlaybackContext | null
): void {
	const el = getAudio();

	playerState.set({
		currentTrack: track,
		queue,
		queueIndex: index,
		isPlaying: true,
		currentTime: 0,
		duration: 0,
		volume: get(playerState).volume,
		playbackContext: context
	});

	if (el) {
		const url = getAudioUrl(track.tid, canPlayOpus());
		el.src = url;
		el.play().catch(() => {
			playerState.update((s) => ({ ...s, isPlaying: false }));
		});
	}

	// If we just started the last track in the local queue, fetch more now
	// so the next chunk is ready by the time this track ends.
	prefetchIfNeeded();
}
