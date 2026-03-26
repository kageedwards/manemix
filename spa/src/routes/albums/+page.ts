import { getPublicAlbums } from '$lib/api/client';
import type { AlbumSummary } from '$lib/types/index.js';

export async function load(): Promise<{ albums: AlbumSummary[] }> {
	const albums = await getPublicAlbums();
	return { albums };
}
