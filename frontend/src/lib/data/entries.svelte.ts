import { apiFetch } from '$lib/api/client';
import { workspaceState } from '$lib/workspace/state.svelte';
import type { JSend } from '$lib/tabs/types';
import type { Entry } from './types';

const state = $state<{ data: Entry[] | null; inFlight: Promise<JSend<Entry[]>> | null }>({
	data: null,
	inFlight: null
});

async function fetchEntries(workspaceId: string): Promise<JSend<Entry[]>> {
	const result = await apiFetch<{ entries: Entry[] }>(`/workspaces/${workspaceId}/entries`);
	if (result.status !== 'success') return result;
	return { status: 'success', data: result.data.entries };
}

export const entries = {
	get value(): Entry[] | null {
		return state.data;
	},
	async load(): Promise<JSend<Entry[]>> {
		if (state.data !== null) return { status: 'success', data: state.data };
		if (state.inFlight) return state.inFlight;
		const promise = fetchEntries(workspaceState.id()).then((result) => {
			if (result.status === 'success') state.data = result.data;
			state.inFlight = null;
			return result;
		});
		state.inFlight = promise;
		return promise;
	},
	invalidate() {
		state.data = null;
		state.inFlight = null;
	},
	hydrate(value: Entry[]) {
		state.data = value;
		state.inFlight = null;
	}
};

export function entryAmount(entry: Entry): number {
	return entry.postings.filter((p) => p.amount > 0).reduce((sum, p) => sum + p.amount, 0);
}
