import { apiFetch } from '$lib/api/client';
import { workspaceState } from '$lib/workspace/state.svelte';
import type { JSend } from '$lib/tabs/types';
import type { RecurringRule } from './loader';
import type { SchedulePresetId } from './schedules';

export type TemplateInput = {
	accountId: string;
	amount: number;
	note?: string;
};

export type CreateRecurringInput = {
	description: string;
	schedule: SchedulePresetId;
	dtstart: string;
	templates: TemplateInput[];
};

type TemplateBody = {
	account_id: string;
	amount: number;
	note?: string;
};

export async function createRecurring(input: CreateRecurringInput): Promise<JSend<RecurringRule>> {
	const templates: TemplateBody[] = input.templates.map((t) => ({
		account_id: t.accountId,
		amount: t.amount,
		...(t.note ? { note: t.note } : {})
	}));

	return apiFetch<RecurringRule>(`/workspaces/${workspaceState.id()}/recurring`, {
		method: 'POST',
		body: JSON.stringify({
			description: input.description,
			schedule: input.schedule,
			dtstart: input.dtstart,
			templates
		})
	});
}

export function deleteRecurring(id: string): Promise<JSend<null>> {
	return apiFetch<null>(`/workspaces/${workspaceState.id()}/recurring/${id}`, {
		method: 'DELETE'
	});
}

export function createRecurringRun(): Promise<JSend<{ posted: number; at: string }>> {
	return apiFetch<{ posted: number; at: string }>(
		`/workspaces/${workspaceState.id()}/recurring/runs`,
		{ method: 'POST', body: '{}' }
	);
}
