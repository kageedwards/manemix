import { getFavorites, ApiError } from '$lib/api/client';
import { error } from '@sveltejs/kit';
import type { Track } from '$lib/types/index.js';

export async function load({ params }: { params: { uid: string } }): Promise<{ tracks: Track[]; uid: number }> {
	try {
		const tracks = await getFavorites(Number(params.uid));
		return { tracks, uid: Number(params.uid) };
	} catch (err) {
		if (err instanceof ApiError) {
			throw error(err.status, err.message);
		}
		throw error(500, 'Failed to load favorites');
	}
}
