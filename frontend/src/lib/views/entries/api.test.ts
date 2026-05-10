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

	it('builds DR/CR postings from single-leg input', async () => {
		const calls = mockFetch({
			status: 201,
			body: { status: 'success', data: { id: 'e1', date: '2026-05-10', description: 'rent', postings: [] } }
		});

		await createEntry({
			date: '2026-05-10',
			description: 'rent',
			debitAccountId: 'acct-rent',
			creditAccountId: 'acct-cash',
			amountCents: 120000,
			note: 'may'
		});

		expect(calls).toHaveLength(1);
		expect(calls[0].init.method).toBe('POST');
		const body = JSON.parse(calls[0].init.body as string);
		expect(body.date).toBe('2026-05-10');
		expect(body.description).toBe('rent');
		expect(body.postings).toEqual([
			{ account_id: 'acct-rent', amount: 120000, note: 'may' },
			{ account_id: 'acct-cash', amount: -120000, note: 'may' }
		]);
	});

	it('omits note when empty', async () => {
		const calls = mockFetch({
			status: 201,
			body: { status: 'success', data: { id: 'e1', date: '2026-05-10', description: 'x', postings: [] } }
		});

		await createEntry({
			date: '2026-05-10',
			description: 'x',
			debitAccountId: 'a',
			creditAccountId: 'b',
			amountCents: 500
		});

		const body = JSON.parse(calls[0].init.body as string);
		expect(body.postings[0].note).toBeUndefined();
		expect(body.postings[1].note).toBeUndefined();
	});

	it('returns JSend success envelope on 201', async () => {
		mockFetch({
			status: 201,
			body: { status: 'success', data: { id: 'e1', date: '2026-05-10', description: 'x', postings: [] } }
		});

		const result = await createEntry({
			date: '2026-05-10',
			description: 'x',
			debitAccountId: 'a',
			creditAccountId: 'b',
			amountCents: 100
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
			debitAccountId: 'a',
			creditAccountId: 'b',
			amountCents: 100
		});

		expect(result.status).toBe('fail');
		if (result.status === 'fail') {
			expect(result.data).toEqual({ description: 'is required' });
		}
	});
});
