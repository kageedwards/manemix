import {
	getFeaturedTracks,
	getLatestTracks,
	getTicker,
	getRecentEvents
} from '$lib/api/client';
import type { Track, TickerItem, EventItem } from '$lib/types/index.js';

export async function load(): Promise<{
	featured: Track[];
	latest: Track[];
	ticker: TickerItem[];
	events: EventItem[];
}> {
	const [featured, latest, ticker, events] = await Promise.all([
		getFeaturedTracks().catch(() => []),
		getLatestTracks().catch(() => []),
		getTicker().catch(() => []),
		getRecentEvents().catch(() => [])
	]);
	return { featured, latest, ticker, events };
}
