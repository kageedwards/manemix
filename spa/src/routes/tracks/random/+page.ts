import { getRandomTracks } from '$lib/api/client';
import type { Track } from '$lib/types/index.js';

export async function load(): Promise<{ tracks: Track[] }> {
	const tracks = await getRandomTracks();
	return { tracks };
}
