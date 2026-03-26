import { apiFetch } from '$lib/api/client';
import type { Track } from '$lib/types/index.js';

export async function load({ url }: { url: URL }): Promise<{ tracks: Track[]; page: number }> {
	const page = Number(url.searchParams.get('p') ?? '1');
	const tracks = await apiFetch<Track[]>(`/tracks/latest?p=${page}`);
	return { tracks, page };
}
