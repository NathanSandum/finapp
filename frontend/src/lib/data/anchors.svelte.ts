import type { JSend } from '$lib/tabs/types';
import type { Anchors } from './types';
import * as backend from '$lib/acl/anchors';

const state = $state<{ value: Anchors | null; inFlight: Promise<JSend<Anchors>> | null }>({
	value: null,
	inFlight: null
});

export const anchors = {
	get value(): Anchors | null {
		return state.value;
	},
	async load(): Promise<JSend<Anchors>> {
		if (state.value !== null) return { status: 'success', data: state.value };
		if (state.inFlight) return state.inFlight;
		const promise = backend.fetchAll().then((result) => {
			if (result.status === 'success') state.value = result.data;
			state.inFlight = null;
			return result;
		});
		state.inFlight = promise;
		return promise;
	},
	hydrate(value: Anchors) {
		state.value = value;
		state.inFlight = null;
	},
	clear() {
		state.value = null;
		state.inFlight = null;
	}
};
