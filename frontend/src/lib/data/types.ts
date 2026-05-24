import type { SchedulePresetId } from '$lib/views/recurring/schedules';

export type Posting = {
	id: string;
	account_id: string;
	amount: number;
	note: string;
};

export type Entry = {
	id: string;
	date: string;
	description: string;
	postings: Posting[];
};

export type AccountRole =
	| 'assets_root'
	| 'liabilities_root'
	| 'equity_root'
	| 'revenue_root'
	| 'expenses_root'
	| 'cash'
	| 'opening_equity';

export type Account = {
	id: string;
	parent_id: string | null;
	name: string;
	account_type: string;
	currency: string;
	role: AccountRole | null;
};

export type Anchors = {
	assets: Account;
	liabilities: Account;
	equity: Account;
	revenue: Account;
	expenses: Account;
	cash: Account;
	opening_equity: Account;
};

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
