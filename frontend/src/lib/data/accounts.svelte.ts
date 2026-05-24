import type { JSend } from '$lib/tabs/types';
import type { Account } from './types';
import * as backend from '$lib/acl/accounts';
import { invalidateDependentTabs } from './invalidate';

const state = $state<{ list: Account[] | null; inFlight: Promise<JSend<Account[]>> | null }>({
	list: null,
	inFlight: null
});

export const accounts = {
	get value(): Account[] | null {
		return state.list;
	},
	async load(): Promise<JSend<Account[]>> {
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
	hydrate(list: Account[]) {
		state.list = list;
		state.inFlight = null;
	},
	clear() {
		state.list = null;
		state.inFlight = null;
	},
	async create(input: backend.CreateInput): Promise<JSend<Account>> {
		const result = await backend.create(input);
		if (result.status === 'success') {
			state.list = [...(state.list ?? []), result.data];
			invalidateDependentTabs();
		}
		return result;
	},
	async rename(id: string, name: string): Promise<JSend<Account>> {
		const result = await backend.rename(id, name);
		if (result.status === 'success' && state.list !== null) {
			state.list = state.list.map((a) => (a.id === id ? result.data : a));
			invalidateDependentTabs();
		}
		return result;
	},
	async remove(id: string): Promise<JSend<null>> {
		const result = await backend.remove(id);
		if (result.status === 'success' && state.list !== null) {
			state.list = state.list.filter((a) => a.id !== id);
			invalidateDependentTabs();
		}
		return result;
	}
};
