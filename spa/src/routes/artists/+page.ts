import { getArtists } from '$lib/api/client';
import type { Artist } from '$lib/types/index.js';

export async function load(): Promise<{ artists: Artist[] }> {
	const artists = await getArtists();
	return { artists };
}
