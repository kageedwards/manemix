/**
 * Lightweight i18n module for the Manemix SPA.
 *
 * Uses JSON locale files with a Svelte writable store.
 * English is the default and only locale at launch.
 * New locales can be added by placing a JSON file in this directory
 * and adding the locale code to `availableLocales`.
 */
import { writable, derived } from 'svelte/store';
import en from './en.json';

/** All message keys available in locale files */
export type MessageKey = keyof typeof en;

/** Map of locale code → messages */
const locales: Record<string, Record<string, string>> = {
	en
};

/** Locale codes that have been loaded */
export const availableLocales: string[] = Object.keys(locales);

/** Current active locale code */
export const locale = writable<string>(detectLocale());

/** Reactive translations record derived from the active locale */
export const translations = derived(locale, ($locale) => {
	return locales[$locale] ?? locales['en'];
});

/**
 * Translate a message key using the current locale.
 * Falls back to English if the key is missing in the active locale.
 */
export function t(messages: Record<string, string>, key: MessageKey): string {
	return messages[key] ?? locales['en'][key] ?? key;
}

/**
 * Reactive translate function — subscribe with $tt in components.
 * Usage: {$tt('nav_home')} — no need to import translations separately.
 */
export const tt = derived(translations, ($msgs) => {
	return (key: MessageKey): string => $msgs[key] ?? locales['en'][key] ?? key;
});

/**
 * Detect the user's preferred locale from `navigator.language`.
 * Returns the best matching available locale, or 'en' as fallback.
 */
function detectLocale(): string {
	if (typeof navigator === 'undefined') return 'en';

	const browserLang = navigator.language;

	// Exact match (e.g. "en" or "fr")
	if (locales[browserLang]) return browserLang;

	// Match base language (e.g. "en-US" → "en")
	const base = browserLang.split('-')[0];
	if (locales[base]) return base;

	return 'en';
}

/**
 * Set the active locale. Falls back to 'en' if the locale is not available.
 */
export function setLocale(code: string): void {
	if (locales[code]) {
		locale.set(code);
	} else {
		locale.set('en');
	}
}
