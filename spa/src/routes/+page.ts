import {
	getFeaturedTracks,
	getLatestTracks,
	getTicker,
	getRecentEvents,
	apiFetch
} from '$lib/api/client';
import type { Track, TickerItem, EventItem } from '$lib/types/index.js';

interface TagItem {
	tag: string;
	count: number;
}

export async function load(): Promise<{
	featured: Track[];
	latest: Track[];
	ticker: TickerItem[];
	events: EventItem[];
	tags: TagItem[];
}> {
	const [featured, latest, ticker, events, tags] = await Promise.all([
		getFeaturedTracks().catch(() => []),
		getLatestTracks().catch(() => []),
		getTicker().catch(() => []),
		getRecentEvents().catch(() => []),
		apiFetch<TagItem[]>('/tags').catch(() => [])
	]);
	return { featured, latest, ticker, events, tags };
}
