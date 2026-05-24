import type { JSend } from '$lib/tabs/types';
import type { RecurringRule } from './types';
import * as backend from '$lib/acl/recurring';
import { invalidateDependentTabs } from './invalidate';

const state = $state<{
	list: RecurringRule[] | null;
	inFlight: Promise<JSend<RecurringRule[]>> | null;
}>({
	list: null,
	inFlight: null
});

export const recurring = {
	get value(): RecurringRule[] | null {
		return state.list;
	},
	async load(): Promise<JSend<RecurringRule[]>> {
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
	hydrate(list: RecurringRule[]) {
		state.list = list;
		state.inFlight = null;
	},
	clear() {
		state.list = null;
		state.inFlight = null;
	},
	async create(input: backend.CreateInput): Promise<JSend<RecurringRule>> {
		const result = await backend.create(input);
		if (result.status === 'success') {
			state.list = [...(state.list ?? []), result.data];
		}
		return result;
	},
	async remove(id: string): Promise<JSend<null>> {
		const result = await backend.remove(id);
		if (result.status === 'success' && state.list !== null) {
			state.list = state.list.filter((r) => r.id !== id);
		}
		return result;
	},
	async runScheduled(): Promise<JSend<backend.RunResult>> {
		const result = await backend.runScheduled();
		if (result.status === 'success' && result.data.posted > 0) {
			invalidateDependentTabs();
		}
		return result;
	}
};
