import { describe, it, expect, vi } from 'vitest';
import type { JSend } from '$lib/tabs/types';

import { cache } from './cache.svelte';

const success = <T>(data: T): JSend<T> => ({ status: 'success', data });

function freshKey(prefix = 'k'): string {
	return `${prefix}-${Math.random().toString(36).slice(2)}`;
}

describe('cache', () => {
	it('returns undefined for unknown key', () => {
		expect(cache.get(freshKey())).toBeUndefined();
	});

	it('stores result of load and returns it on subsequent get', async () => {
		const key = freshKey();
		const result = await cache.load(key, async () => success(42));
		expect(result).toEqual(success(42));
		expect(cache.get(key)).toEqual(success(42));
	});

	it('does not refetch when value is cached and load is called again', async () => {
		const key = freshKey();
		const fetcher = vi.fn(async () => success('first'));

		await cache.load(key, fetcher);
		expect(fetcher).toHaveBeenCalledTimes(1);

		// Cache hit path: load() should still return the cached value without
		// invoking the fetcher again.
		const second = await cache.load(key, async () => success('second'));
		expect(second).toEqual(success('first'));
	});

	it('dedups concurrent loads for the same key', async () => {
		const key = freshKey();
		let resolveFetch!: (value: JSend<string>) => void;
		const fetcher = vi.fn(
			() => new Promise<JSend<string>>((resolve) => (resolveFetch = resolve))
		);

		const a = cache.load(key, fetcher);
		const b = cache.load(key, fetcher);

		expect(fetcher).toHaveBeenCalledTimes(1);

		resolveFetch(success('shared'));
		expect(await a).toEqual(success('shared'));
		expect(await b).toEqual(success('shared'));
	});

	it('refetches after invalidate', async () => {
		const key = freshKey();
		let count = 0;
		const fetcher = async () => success(++count);

		await cache.load(key, fetcher);
		expect(cache.get(key)).toEqual(success(1));

		cache.invalidate(key);
		expect(cache.get(key)).toBeUndefined();

		await cache.load(key, fetcher);
		expect(cache.get(key)).toEqual(success(2));
	});

	it('keeps separate keys independent', async () => {
		const a = freshKey('a');
		const b = freshKey('b');

		await cache.load(a, async () => success('A'));
		await cache.load(b, async () => success('B'));

		expect(cache.get(a)).toEqual(success('A'));
		expect(cache.get(b)).toEqual(success('B'));

		cache.invalidate(a);
		expect(cache.get(a)).toBeUndefined();
		expect(cache.get(b)).toEqual(success('B'));
	});

	it('invalidateAll clears every key', async () => {
		const a = freshKey('a');
		const b = freshKey('b');
		await cache.load(a, async () => success('A'));
		await cache.load(b, async () => success('B'));

		cache.invalidateAll();

		expect(cache.get(a)).toBeUndefined();
		expect(cache.get(b)).toBeUndefined();
	});

	it('stores fail and error envelopes too', async () => {
		const key = freshKey();
		const failResult: JSend<unknown> = {
			status: 'fail',
			data: { email: 'is required' }
		};
		await cache.load(key, async () => failResult);
		expect(cache.get(key)).toEqual(failResult);

		const key2 = freshKey();
		const errorResult: JSend<unknown> = { status: 'error', message: 'down' };
		await cache.load(key2, async () => errorResult);
		expect(cache.get(key2)).toEqual(errorResult);
	});
});
