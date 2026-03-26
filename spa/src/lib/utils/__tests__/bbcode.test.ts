import { describe, it, expect } from 'vitest';
import { renderBBCode } from '../bbcode';

describe('renderBBCode', () => {
	it('returns empty string for null/undefined/empty input', () => {
		expect(renderBBCode('')).toBe('');
		expect(renderBBCode(null as unknown as string)).toBe('');
		expect(renderBBCode(undefined as unknown as string)).toBe('');
	});

	it('returns plain text unchanged', () => {
		expect(renderBBCode('hello world')).toBe('hello world');
	});

	it('renders [b] as <strong>', () => {
		expect(renderBBCode('[b]bold[/b]')).toBe('<strong>bold</strong>');
	});

	it('renders [i] as <em>', () => {
		expect(renderBBCode('[i]italic[/i]')).toBe('<em>italic</em>');
	});

	it('renders [u] as <u>', () => {
		expect(renderBBCode('[u]underline[/u]')).toBe('<u>underline</u>');
	});

	it('renders [url=href]text[/url] as anchor with target and rel', () => {
		expect(renderBBCode('[url=http://example.com]click[/url]')).toBe(
			'<a href="http://example.com" target="_blank" rel="noopener noreferrer">click</a>'
		);
	});

	it('renders [url]href[/url] as anchor with href as text', () => {
		expect(renderBBCode('[url]http://example.com[/url]')).toBe(
			'<a href="http://example.com" target="_blank" rel="noopener noreferrer">http://example.com</a>'
		);
	});

	it('renders [img] as img with lazy loading', () => {
		expect(renderBBCode('[img]http://example.com/pic.png[/img]')).toBe(
			'<img src="http://example.com/pic.png" alt="" loading="lazy" />'
		);
	});

	it('handles nested tags', () => {
		expect(renderBBCode('[b][i]bold italic[/i][/b]')).toBe(
			'<strong><em>bold italic</em></strong>'
		);
	});

	it('handles case-insensitive tags', () => {
		expect(renderBBCode('[B]bold[/B]')).toBe('<strong>bold</strong>');
		expect(renderBBCode('[I]italic[/I]')).toBe('<em>italic</em>');
		expect(renderBBCode('[U]underline[/U]')).toBe('<u>underline</u>');
	});

	it('HTML-escapes input to prevent XSS', () => {
		expect(renderBBCode('<script>alert("xss")</script>')).toBe(
			'&lt;script&gt;alert(&quot;xss&quot;)&lt;/script&gt;'
		);
	});

	it('HTML-escapes content inside BBCode tags', () => {
		expect(renderBBCode('[b]<em>not real html</em>[/b]')).toBe(
			'<strong>&lt;em&gt;not real html&lt;/em&gt;</strong>'
		);
	});

	it('escapes ampersands and quotes', () => {
		expect(renderBBCode('Tom & Jerry\'s "show"')).toBe(
			'Tom &amp; Jerry&#39;s &quot;show&quot;'
		);
	});

	it('handles multiple tags in sequence', () => {
		expect(renderBBCode('[b]one[/b] [i]two[/i] [u]three[/u]')).toBe(
			'<strong>one</strong> <em>two</em> <u>three</u>'
		);
	});

	it('handles multiline content inside tags', () => {
		expect(renderBBCode('[b]line1\nline2[/b]')).toBe('<strong>line1\nline2</strong>');
	});
});
