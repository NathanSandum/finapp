import { apiFetch } from '$lib/api/client';
import { workspaceState } from '$lib/workspace/state.svelte';
import type { JSend } from '$lib/tabs/types';
import type { Anchors } from './types';

const state = $state<{ data: Anchors | null; inFlight: Promise<JSend<Anchors>> | null }>({
	data: null,
	inFlight: null
});

function fetchAnchors(workspaceId: string): Promise<JSend<Anchors>> {
	return apiFetch<Anchors>(`/workspaces/${workspaceId}/anchors`);
}

export const anchors = {
	get value(): Anchors | null {
		return state.data;
	},
	async load(): Promise<JSend<Anchors>> {
		if (state.data !== null) return { status: 'success', data: state.data };
		if (state.inFlight) return state.inFlight;
		const promise = fetchAnchors(workspaceState.id()).then((result) => {
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
	hydrate(value: Anchors) {
		state.data = value;
		state.inFlight = null;
	}
};
