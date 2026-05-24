import { apiFetch } from '$lib/api/client';
import { workspaceState } from '$lib/workspace/state.svelte';
import type { JSend } from '$lib/tabs/types';
import type { SchedulePresetId } from '$lib/views/recurring/schedules';

export type RecurringTemplate = {
	id: string;
	rule: string;
	account: string;
	amount: number;
	note: string;
};

export type RecurringRule = {
	id: string;
	workspace: string;
	description: string;
	schedule: SchedulePresetId | null;
	dtstart: string;
	last_fired_on: string | null;
	deleted_at: string | null;
	created_at: string;
	updated_at: string;
	templates: RecurringTemplate[];
	next_pending: string | null;
};

const state = $state<{
	data: RecurringRule[] | null;
	inFlight: Promise<JSend<RecurringRule[]>> | null;
}>({
	data: null,
	inFlight: null
});

async function fetchRecurring(workspaceId: string): Promise<JSend<RecurringRule[]>> {
	const result = await apiFetch<{ recurring_rules: RecurringRule[] }>(
		`/workspaces/${workspaceId}/recurring`
	);
	if (result.status !== 'success') return result;
	return { status: 'success', data: result.data.recurring_rules };
}

export const recurring = {
	get value(): RecurringRule[] | null {
		return state.data;
	},
	async load(): Promise<JSend<RecurringRule[]>> {
		if (state.data !== null) return { status: 'success', data: state.data };
		if (state.inFlight) return state.inFlight;
		const promise = fetchRecurring(workspaceState.id()).then((result) => {
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
	hydrate(value: RecurringRule[]) {
		state.data = value;
		state.inFlight = null;
	}
};
