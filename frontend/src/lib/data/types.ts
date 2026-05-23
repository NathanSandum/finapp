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
