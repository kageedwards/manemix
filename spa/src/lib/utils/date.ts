/**
 * Format a raw timestamp string into a human-readable date.
 * Full format: "May 15, 2026 06:16"
 * Short format (for small screens): "May 15, 2026"
 */

const fullFormatter = new Intl.DateTimeFormat('en-US', {
	year: 'numeric',
	month: 'long',
	day: 'numeric',
	hour: '2-digit',
	minute: '2-digit',
	hour12: false
});

const shortFormatter = new Intl.DateTimeFormat('en-US', {
	year: 'numeric',
	month: 'short',
	day: 'numeric'
});

/**
 * Parse a postgres timestamp string into a Date.
 * Handles both "2026-05-15 06:16:33.931869+00" and ISO formats.
 */
function parseTimestamp(raw: string): Date {
	// Replace space between date and time with 'T' for reliable parsing
	const normalized = raw.replace(' ', 'T').replace('+00', '+00:00');
	return new Date(normalized);
}

/** Full format: "May 15, 2026 06:16" */
export function formatDate(raw: string): string {
	if (!raw) return '';
	try {
		return fullFormatter.format(parseTimestamp(raw));
	} catch {
		return raw;
	}
}

/** Short format: "May 15, 2026" */
export function formatDateShort(raw: string): string {
	if (!raw) return '';
	try {
		return shortFormatter.format(parseTimestamp(raw));
	} catch {
		return raw;
	}
}
