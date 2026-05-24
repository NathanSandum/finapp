import { apiFetch } from './client';
import { workspaceState } from '$lib/workspace/state.svelte';
import type { JSend } from '$lib/tabs/types';
import type { Account, AccountRole, Anchors, Entry, Posting, RecurringRule } from '$lib/data/types';
import type { SchedulePresetId } from '$lib/views/recurring/schedules';

type WirePosting = { id: string; account_id: string; amount: number; note: string };
type WireEntry = { id: string; date: string; description: string; postings: WirePosting[] };
type WireAccount = {
	id: string;
	parent_id: string | null;
	name: string;
	account_type: string;
	currency: string;
	role: AccountRole | null;
};
type WireTemplate = { id: string; rule: string; account: string; amount: number; note: string };
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

type WireBundle = {
	entries: WireEntry[];
	accounts: WireAccount[];
	anchors: Anchors;
	recurring_rules: WireRule[];
};

export type Bundle = {
	entries: Entry[];
	accounts: Account[];
	anchors: Anchors;
	recurringRules: RecurringRule[];
};

function toPosting(w: WirePosting): Posting {
	return { id: w.id, account_id: w.account_id, amount: w.amount, note: w.note };
}
function toEntry(w: WireEntry): Entry {
	return {
		id: w.id,
		date: w.date,
		description: w.description,
		postings: w.postings.map(toPosting)
	};
}
function toAccount(w: WireAccount): Account {
	return {
		id: w.id,
		parent_id: w.parent_id,
		name: w.name,
		account_type: w.account_type,
		currency: w.currency,
		role: w.role
	};
}
function toTemplate(w: WireTemplate) {
	return { id: w.id, rule: w.rule, account: w.account, amount: w.amount, note: w.note };
}
function toRule(w: WireRule): RecurringRule {
	return {
		id: w.id,
		workspace: w.workspace,
		description: w.description,
		schedule: w.schedule,
		dtstart: w.dtstart,
		last_fired_on: w.last_fired_on,
		deleted_at: w.deleted_at,
		created_at: w.created_at,
		updated_at: w.updated_at,
		templates: w.templates.map(toTemplate),
		next_pending: w.next_pending
	};
}

export async function fetchAll(): Promise<JSend<Bundle>> {
	const result = await apiFetch<WireBundle>(`/workspaces/${workspaceState.id()}/bootstrap`);
	if (result.status !== 'success') return result;
	return {
		status: 'success',
		data: {
			entries: result.data.entries.map(toEntry),
			accounts: result.data.accounts.map(toAccount),
			anchors: result.data.anchors,
			recurringRules: result.data.recurring_rules.map(toRule)
		}
	};
}
