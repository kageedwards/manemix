import { apiFetch, ApiError } from '$lib/api/client';
import { error } from '@sveltejs/kit';
import type { Track } from '$lib/types/index.js';

export async function load({ params }: { params: { tag: string } }): Promise<{ tracks: Track[]; tag: string }> {
	try {
		const tracks = await apiFetch<Track[]>(`/tracks/tag/${encodeURIComponent(params.tag)}`);
		return { tracks, tag: params.tag };
	} catch (err) {
		if (err instanceof ApiError) {
			throw error(err.status, err.message);
		}
		throw error(500, 'Failed to load tracks');
	}
}
