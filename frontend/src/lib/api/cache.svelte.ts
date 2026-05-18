import type { JSend } from '$lib/tabs/types';

const store: Record<string, JSend<unknown>> = $state({});
const inFlight = new Map<string, Promise<JSend<unknown>>>();

export const cache = {
	get<T>(key: string): JSend<T> | undefined {
		return store[key] as JSend<T> | undefined;
	},

	async load<T>(key: string, fetcher: () => Promise<JSend<T>>): Promise<JSend<T>> {
		const cached = store[key];
		if (cached !== undefined) return cached as JSend<T>;

		const existing = inFlight.get(key);
		if (existing) return existing as Promise<JSend<T>>;

		const promise = fetcher().then((result) => {
			store[key] = result;
			inFlight.delete(key);
			return result;
		});
		inFlight.set(key, promise as Promise<JSend<unknown>>);
		return promise;
	},

	async refresh<T>(key: string, fetcher: () => Promise<JSend<T>>): Promise<JSend<T>> {
		const existing = inFlight.get(key);
		if (existing) return existing as Promise<JSend<T>>;

		const promise = fetcher().then((result) => {
			store[key] = result;
			inFlight.delete(key);
			return result;
		});
		inFlight.set(key, promise as Promise<JSend<unknown>>);
		return promise;
	},

	invalidate(key: string) {
		delete store[key];
		inFlight.delete(key);
	},

	invalidateAll() {
		for (const key of Object.keys(store)) delete store[key];
		inFlight.clear();
	},

	invalidateAllExcept(keep: string) {
		for (const key of Object.keys(store)) {
			if (key !== keep) delete store[key];
		}
		for (const key of inFlight.keys()) {
			if (key !== keep) inFlight.delete(key);
		}
	}
};
