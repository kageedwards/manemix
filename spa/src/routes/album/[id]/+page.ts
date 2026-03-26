import { getAlbum } from '$lib/api/client';
import type { AlbumData } from '$lib/types/index.js';

export async function load({ params }: { params: { id: string } }): Promise<{ data: AlbumData }> {
	const data = await getAlbum(Number(params.id));
	return { data };
}
