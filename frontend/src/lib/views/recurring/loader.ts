import { apiFetch } from '$lib/api/client';
import { workspaceState } from '$lib/workspace/state.svelte';
import type { JSend } from '$lib/tabs/types';
import type { Account, Anchors } from '$lib/views/entries/loader';
import type { SchedulePresetId } from './schedules';

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

export type RecurringData = {
	rules: RecurringRule[];
	accounts: Account[];
	anchors: Anchors;
};

export async function load(): Promise<JSend<RecurringData>> {
	const id = workspaceState.id();
	const [rulesResult, accountsResult, anchorsResult] = await Promise.all([
		apiFetch<{ recurring_rules: RecurringRule[] }>(`/workspaces/${id}/recurring`),
		apiFetch<{ accounts: Account[] }>(`/workspaces/${id}/accounts`),
		apiFetch<Anchors>(`/workspaces/${id}/anchors`)
	]);

	if (rulesResult.status !== 'success') return rulesResult;
	if (accountsResult.status !== 'success') return accountsResult;
	if (anchorsResult.status !== 'success') return anchorsResult;

	return {
		status: 'success',
		data: {
			rules: rulesResult.data.recurring_rules,
			accounts: accountsResult.data.accounts,
			anchors: anchorsResult.data
		}
	};
}
