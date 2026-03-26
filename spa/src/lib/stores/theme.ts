import { writable } from 'svelte/store';

export type Theme = 'dark' | 'light';

const STORAGE_KEY = 'theme';
const DEFAULT_THEME: Theme = 'dark';

const THEME_MAP: Record<Theme, string> = {
	dark: 'manemix_dark',
	light: 'manemix_light'
};

function getInitialTheme(): Theme {
	if (typeof window !== 'undefined') {
		const stored = localStorage.getItem(STORAGE_KEY);
		if (stored === 'dark' || stored === 'light') {
			return stored;
		}
	}
	return DEFAULT_THEME;
}

function applyTheme(theme: Theme): void {
	if (typeof window !== 'undefined') {
		document.documentElement.setAttribute('data-theme', THEME_MAP[theme]);
	}
}

function persistTheme(theme: Theme): void {
	if (typeof window !== 'undefined') {
		localStorage.setItem(STORAGE_KEY, theme);
	}
}

const initialTheme = getInitialTheme();
applyTheme(initialTheme);

export const theme = writable<Theme>(initialTheme);

theme.subscribe((value) => {
	applyTheme(value);
	persistTheme(value);
});

export function toggleTheme(): void {
	theme.update((current) => (current === 'dark' ? 'light' : 'dark'));
}
