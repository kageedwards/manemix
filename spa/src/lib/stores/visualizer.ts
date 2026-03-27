import { writable } from 'svelte/store';

const STORAGE_KEY = 'visualizer';
const DEFAULT: boolean = true;

function getInitial(): boolean {
	if (typeof window !== 'undefined') {
		const stored = localStorage.getItem(STORAGE_KEY);
		if (stored === 'true' || stored === 'false') {
			return stored === 'true';
		}
	}
	return DEFAULT;
}

const initial = getInitial();

export const visualizerEnabled = writable<boolean>(initial);

visualizerEnabled.subscribe((value) => {
	if (typeof window !== 'undefined') {
		localStorage.setItem(STORAGE_KEY, String(value));
	}
});

export function toggleVisualizer(): void {
	visualizerEnabled.update((current) => !current);
}
