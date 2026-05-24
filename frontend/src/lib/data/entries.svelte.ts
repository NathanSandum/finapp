import type { JSend } from '$lib/tabs/types';
import type { Entry } from './types';
import * as backend from '$lib/acl/entries';
import { invalidateDependentTabs } from './invalidate';

const state = $state<{ list: Entry[] | null; inFlight: Promise<JSend<Entry[]>> | null }>({
	list: null,
	inFlight: null
});

function sortDesc(list: Entry[]): Entry[] {
	return [...list].sort((a, b) => (a.date < b.date ? 1 : a.date > b.date ? -1 : 0));
}

export const entries = {
	get value(): Entry[] | null {
		return state.list;
	},
	async load(): Promise<JSend<Entry[]>> {
		if (state.list !== null) return { status: 'success', data: state.list };
		if (state.inFlight) return state.inFlight;
		const promise = backend.fetchAll().then((result) => {
			if (result.status === 'success') state.list = result.data;
			state.inFlight = null;
			return result;
		});
		state.inFlight = promise;
		return promise;
	},
	hydrate(list: Entry[]) {
		state.list = list;
		state.inFlight = null;
	},
	clear() {
		state.list = null;
		state.inFlight = null;
	},
	async create(input: backend.CreateInput): Promise<JSend<Entry>> {
		const result = await backend.create(input);
		if (result.status === 'success') {
			state.list = sortDesc([...(state.list ?? []), result.data]);
			invalidateDependentTabs();
		}
		return result;
	},
	async remove(id: string): Promise<JSend<null>> {
		const result = await backend.remove(id);
		if (result.status === 'success' && state.list !== null) {
			state.list = state.list.filter((e) => e.id !== id);
			invalidateDependentTabs();
		}
		return result;
	}
};

export function entryAmount(entry: Entry): number {
	return entry.postings.filter((p) => p.amount > 0).reduce((sum, p) => sum + p.amount, 0);
}
