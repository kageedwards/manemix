/**
 * Track audio readiness store.
 * Monitors tracks that are still transcoding and polls until ready.
 */
import { writable, get } from 'svelte/store';
import { apiFetch } from '$lib/api/client';

interface TrackStatusEntry {
	ready: boolean;
	polling: boolean;
}

const statusMap = writable<Record<number, TrackStatusEntry>>({});
const timers = new Map<number, ReturnType<typeof setInterval>>();

const POLL_INTERVAL = 5000; // 5 seconds

export const trackStatuses = { subscribe: statusMap.subscribe };

/** Check if a track is ready to play. Returns true if ready or unknown (optimistic). */
export function isTrackReady(tid: number): boolean {
	const map = get(statusMap);
	return map[tid]?.ready ?? true; // optimistic default
}

/** Register a track for status monitoring. Fetches status once, polls if not ready. */
export function monitorTrack(tid: number): void {
	const map = get(statusMap);
	if (map[tid] !== undefined) return; // already monitoring

	// Mark as unknown initially
	statusMap.update((m) => ({ ...m, [tid]: { ready: true, polling: false } }));

	checkStatus(tid);
}

/** Stop monitoring a track (e.g. when its card leaves the DOM). */
export function unmonitorTrack(tid: number): void {
	const timer = timers.get(tid);
	if (timer) {
		clearInterval(timer);
		timers.delete(tid);
	}
}

async function checkStatus(tid: number) {
	try {
		const data = await apiFetch<{ ready: boolean }>(`/track/${tid}/status`);
		statusMap.update((m) => ({ ...m, [tid]: { ready: data.ready, polling: !data.ready } }));

		if (!data.ready && !timers.has(tid)) {
			// Start polling
			const timer = setInterval(async () => {
				try {
					const d = await apiFetch<{ ready: boolean }>(`/track/${tid}/status`);
					if (d.ready) {
						statusMap.update((m) => ({ ...m, [tid]: { ready: true, polling: false } }));
						clearInterval(timer);
						timers.delete(tid);
					}
				} catch {
					// Ignore poll errors
				}
			}, POLL_INTERVAL);
			timers.set(tid, timer);
		}
	} catch {
		// If status check fails, assume ready (don't block playback for old tracks)
	}
}
