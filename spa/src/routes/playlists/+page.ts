import { getPublicPlaylists } from '$lib/api/client';
import type { PlaylistSummary } from '$lib/types/index.js';

export async function load(): Promise<{ playlists: PlaylistSummary[] }> {
	const playlists = await getPublicPlaylists();
	return { playlists };
}
