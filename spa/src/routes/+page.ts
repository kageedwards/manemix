import { getFeaturedTracks, getLatestTracks, getRandomTracks } from '$lib/api/client';
import type { Track } from '$lib/types/index.js';

export async function load(): Promise<{
	featured: Track[];
	latest: Track[];
	random: Track[];
}> {
	const [featured, latest, random] = await Promise.all([
		getFeaturedTracks().catch(() => []),
		getLatestTracks().catch(() => []),
		getRandomTracks().catch(() => [])
	]);
	return { featured, latest, random };
}
