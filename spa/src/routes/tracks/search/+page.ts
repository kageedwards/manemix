import { searchTracks } from '$lib/api/client';
import type { Track } from '$lib/types/index.js';

export async function load({ url }: { url: URL }): Promise<{ tracks: Track[]; query: string }> {
	const q = url.searchParams.get('q') ?? '';
	const tracks = q ? await searchTracks(q) : [];
	return { tracks, query: q };
}
