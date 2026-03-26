import { getPlaylist, ApiError } from '$lib/api/client';
import { error } from '@sveltejs/kit';
import type { PlaylistData } from '$lib/types/index.js';

export async function load({ params }: { params: { id: string } }): Promise<{ data: PlaylistData }> {
	try {
		const data = await getPlaylist(Number(params.id));
		return { data };
	} catch (err) {
		if (err instanceof ApiError) {
			throw error(err.status, err.message);
		}
		throw error(500, 'Failed to load playlist');
	}
}
