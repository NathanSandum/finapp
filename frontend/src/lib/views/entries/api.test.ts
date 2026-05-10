import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { createEntry } from './api';

type FetchArgs = { url: string; init: RequestInit };

function mockFetch(response: { status: number; body: unknown }) {
	const calls: FetchArgs[] = [];
	const fn = vi.fn(async (url: string, init: RequestInit) => {
		calls.push({ url, init });
		return new Response(JSON.stringify(response.body), {
			status: response.status,
			headers: { 'Content-Type': 'application/json' }
		});
	});
	// @ts-expect-error override
	global.fetch = fn;
	return calls;
}

describe('createEntry', () => {
	beforeEach(() => {
		vi.restoreAllMocks();
	});
	afterEach(() => {
		vi.restoreAllMocks();
	});

	it('POSTs date, description, and postings to entries endpoint', async () => {
		const calls = mockFetch({
			status: 201,
			body: { status: 'success', data: { id: 'e1', date: '2026-05-10', description: 'rent', postings: [] } }
		});

		await createEntry({
			date: '2026-05-10',
			description: 'rent',
			postings: [
				{ accountId: 'acct-rent', amount: 120000 },
				{ accountId: 'acct-cash', amount: -120000 }
			]
		});

		expect(calls).toHaveLength(1);
		expect(calls[0].init.method).toBe('POST');
		const body = JSON.parse(calls[0].init.body as string);
		expect(body).toEqual({
			date: '2026-05-10',
			description: 'rent',
			postings: [
				{ account_id: 'acct-rent', amount: 120000 },
				{ account_id: 'acct-cash', amount: -120000 }
			]
		});
	});

	it('attaches note to every posting when provided', async () => {
		const calls = mockFetch({
			status: 201,
			body: { status: 'success', data: { id: 'e1', date: '2026-05-10', description: 'x', postings: [] } }
		});

		await createEntry({
			date: '2026-05-10',
			description: 'x',
			postings: [
				{ accountId: 'a', amount: 500 },
				{ accountId: 'b', amount: -500 }
			],
			note: 'may'
		});

		const body = JSON.parse(calls[0].init.body as string);
		expect(body.postings).toEqual([
			{ account_id: 'a', amount: 500, note: 'may' },
			{ account_id: 'b', amount: -500, note: 'may' }
		]);
	});

	it('omits note key when not provided or empty', async () => {
		const calls = mockFetch({
			status: 201,
			body: { status: 'success', data: { id: 'e1', date: '2026-05-10', description: 'x', postings: [] } }
		});

		await createEntry({
			date: '2026-05-10',
			description: 'x',
			postings: [
				{ accountId: 'a', amount: 1 },
				{ accountId: 'b', amount: -1 }
			],
			note: '  '
		});

		const body = JSON.parse(calls[0].init.body as string);
		expect(body.postings[0].note).toBeUndefined();
	});

	it('returns JSend success envelope on 201', async () => {
		mockFetch({
			status: 201,
			body: { status: 'success', data: { id: 'e1', date: '2026-05-10', description: 'x', postings: [] } }
		});

		const result = await createEntry({
			date: '2026-05-10',
			description: 'x',
			postings: [
				{ accountId: 'a', amount: 1 },
				{ accountId: 'b', amount: -1 }
			]
		});

		expect(result.status).toBe('success');
	});

	it('forwards JSend fail envelope', async () => {
		mockFetch({
			status: 400,
			body: { status: 'fail', data: { description: 'is required' } }
		});

		const result = await createEntry({
			date: '2026-05-10',
			description: '',
			postings: [
				{ accountId: 'a', amount: 1 },
				{ accountId: 'b', amount: -1 }
			]
		});

		expect(result.status).toBe('fail');
		if (result.status === 'fail') {
			expect(result.data).toEqual({ description: 'is required' });
		}
	});
});
