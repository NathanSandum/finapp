import { apiFetch } from './client';
import { workspaceState } from '$lib/workspace/state.svelte';
import type { JSend } from '$lib/tabs/types';
import type { RecurringRule, RecurringTemplate } from '$lib/data/types';
import type { SchedulePresetId } from '$lib/views/recurring/schedules';

type WireTemplate = {
	id: string;
	rule: string;
	account: string;
	amount: number;
	note: string;
};

type WireRule = {
	id: string;
	workspace: string;
	description: string;
	schedule: SchedulePresetId | null;
	dtstart: string;
	last_fired_on: string | null;
	deleted_at: string | null;
	created_at: string;
	updated_at: string;
	templates: WireTemplate[];
	next_pending: string | null;
};

function toTemplate(wire: WireTemplate): RecurringTemplate {
	return { id: wire.id, rule: wire.rule, account: wire.account, amount: wire.amount, note: wire.note };
}

function toRule(wire: WireRule): RecurringRule {
	return {
		id: wire.id,
		workspace: wire.workspace,
		description: wire.description,
		schedule: wire.schedule,
		dtstart: wire.dtstart,
		last_fired_on: wire.last_fired_on,
		deleted_at: wire.deleted_at,
		created_at: wire.created_at,
		updated_at: wire.updated_at,
		templates: wire.templates.map(toTemplate),
		next_pending: wire.next_pending
	};
}

export async function fetchAll(): Promise<JSend<RecurringRule[]>> {
	const result = await apiFetch<{ recurring_rules: WireRule[] }>(
		`/workspaces/${workspaceState.id()}/recurring`
	);
	if (result.status !== 'success') return result;
	return { status: 'success', data: result.data.recurring_rules.map(toRule) };
}

export type TemplateInput = { accountId: string; amount: number; note?: string };
export type CreateInput = {
	description: string;
	schedule: SchedulePresetId;
	dtstart: string;
	templates: TemplateInput[];
};

export async function create(input: CreateInput): Promise<JSend<RecurringRule>> {
	const templates = input.templates.map((t) => ({
		account_id: t.accountId,
		amount: t.amount,
		...(t.note ? { note: t.note } : {})
	}));
	const result = await apiFetch<WireRule>(`/workspaces/${workspaceState.id()}/recurring`, {
		method: 'POST',
		body: JSON.stringify({
			description: input.description,
			schedule: input.schedule,
			dtstart: input.dtstart,
			templates
		})
	});
	if (result.status !== 'success') return result;
	return { status: 'success', data: toRule(result.data) };
}

export function remove(id: string): Promise<JSend<null>> {
	return apiFetch<null>(`/workspaces/${workspaceState.id()}/recurring/${id}`, {
		method: 'DELETE'
	});
}

export type RunResult = { posted: number; at: string };

export function runScheduled(): Promise<JSend<RunResult>> {
	return apiFetch<RunResult>(`/workspaces/${workspaceState.id()}/recurring/runs`, {
		method: 'POST',
		body: '{}'
	});
}
