// ── Account ─────────────────────────────────────────────────────────────────

export type Account = {
	id: string;
	name: string;
	account_type: string;
	parent_id: string | null;
};

// ── Entries ──────────────────────────────────────────────────────────────────

export type PostingInput = {
	account_id: string;
	amount: number;
	note?: string;
};

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
	cleared: boolean;
	postings: Posting[];
};

// ── Reports ─────────────────────────────────────────────────────────────────

export type AccountBalance = {
	account: string;
	name: string;
	balance: number;
};

export type IncomeStatement = {
	start_date: string;
	end_date: string;
	revenue: AccountBalance[];
	expenses: AccountBalance[];
	total_revenue: number;
	total_expenses: number;
	net_income: number;
};

export type BalanceSheet = {
	as_of_date: string;
	assets: AccountBalance[];
	liabilities: AccountBalance[];
	equity: AccountBalance[];
	total_assets: number;
	total_liabilities: number;
	total_equity: number;
	net_income: number;
	total_liabilities_and_equity: number;
};
