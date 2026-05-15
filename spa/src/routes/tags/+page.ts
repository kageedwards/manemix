import { apiFetch } from '$lib/api/client';

interface TagItem {
	tag: string;
	count: number;
}

export async function load(): Promise<{ tags: TagItem[] }> {
	const tags = await apiFetch<TagItem[]>('/tags').catch(() => []);
	return { tags };
}
