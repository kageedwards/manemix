import { getTrack, ApiError } from '$lib/api/client';
import { error } from '@sveltejs/kit';
import type { ExtendedTrack } from '$lib/types/index.js';

export async function load({ params }: { params: { tid: string } }): Promise<{ track: ExtendedTrack }> {
	try {
		const track = await getTrack(Number(params.tid));
		return { track };
	} catch (err) {
		if (err instanceof ApiError) {
			throw error(err.status, err.message);
		}
		throw error(500, 'Failed to load track');
	}
}
