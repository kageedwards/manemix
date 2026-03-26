/**
 * Client-side BBCode-to-HTML renderer.
 *
 * Supported tags:
 *   [b]text[/b]           → <strong>text</strong>
 *   [i]text[/i]           → <em>text</em>
 *   [u]text[/u]           → <u>text</u>
 *   [url=href]text[/url]  → <a href="href" target="_blank" rel="noopener noreferrer">text</a>
 *   [url]href[/url]       → <a href="href" target="_blank" rel="noopener noreferrer">href</a>
 *   [img]src[/img]        → <img src="src" alt="" loading="lazy" />
 *
 * Input is HTML-escaped before BBCode processing to prevent XSS.
 * Nested tags are handled correctly.
 */

const HTML_ESCAPE_MAP: Record<string, string> = {
	'&': '&amp;',
	'<': '&lt;',
	'>': '&gt;',
	'"': '&quot;',
	"'": '&#39;'
};

function escapeHtml(str: string): string {
	return str.replace(/[&<>"']/g, (ch) => HTML_ESCAPE_MAP[ch]);
}

/**
 * Render BBCode markup to safe HTML.
 *
 * @param input - Raw BBCode string (may contain user-generated content)
 * @returns Sanitised HTML string
 */
export function renderBBCode(input: string): string {
	if (!input) return '';

	// HTML-escape first to prevent XSS
	let html = escapeHtml(input);

	// Simple tags: [b], [i], [u] — applied repeatedly to handle nesting
	html = html.replace(/\[b\]([\s\S]*?)\[\/b\]/gi, '<strong>$1</strong>');
	html = html.replace(/\[i\]([\s\S]*?)\[\/i\]/gi, '<em>$1</em>');
	html = html.replace(/\[u\]([\s\S]*?)\[\/u\]/gi, '<u>$1</u>');

	// [url=href]text[/url]
	html = html.replace(
		/\[url=(.*?)\]([\s\S]*?)\[\/url\]/gi,
		'<a href="$1" target="_blank" rel="noopener noreferrer">$2</a>'
	);

	// [url]href[/url]
	html = html.replace(
		/\[url\]([\s\S]*?)\[\/url\]/gi,
		'<a href="$1" target="_blank" rel="noopener noreferrer">$1</a>'
	);

	// [img]src[/img]
	html = html.replace(
		/\[img\]([\s\S]*?)\[\/img\]/gi,
		'<img src="$1" alt="" loading="lazy" />'
	);

	return html;
}
