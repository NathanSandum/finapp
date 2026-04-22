const BASE = '/api';

let token = $state<string | null>(null);

export function setToken(t: string | null) {
	token = t;
	if (t) {
		localStorage.setItem('token', t);
	} else {
		localStorage.removeItem('token');
	}
}

export function loadToken() {
	token = localStorage.getItem('token');
}

export function getToken() {
	return token;
}

async function request(method: string, path: string, body?: unknown): Promise<Response> {
	const headers: Record<string, string> = {};
	if (token) headers['Authorization'] = `Bearer ${token}`;
	if (body) headers['Content-Type'] = 'application/json';

	return fetch(`${BASE}${path}`, {
		method,
		headers,
		body: body ? JSON.stringify(body) : undefined
	});
}

// ── Auth ────────────────────────────────────────────────────────────────────

export async function register(email: string, displayName: string, password: string) {
	const res = await request('POST', '/auth/register', {
		email,
		display_name: displayName,
		password
	});
	if (!res.ok) throw await res.json();
	return res.json();
}

export async function login(email: string, password: string) {
	const res = await request('POST', '/auth/login', { email, password });
	if (!res.ok) throw await res.json();
	return res.json();
}

export async function logout() {
	await request('POST', '/auth/logout');
	setToken(null);
}

// ── Workspaces ──────────────────────────────────────────────────────────────

export async function listWorkspaces() {
	const res = await request('GET', '/workspaces');
	if (!res.ok) throw await res.json();
	return res.json();
}

export async function createWorkspace(name: string) {
	const res = await request('POST', '/workspaces', { name });
	if (!res.ok) throw await res.json();
	return res.json();
}

// ── Accounts ────────────────────────────────────────────────────────────────

export async function listAccounts(workspace: string) {
	const res = await request('GET', `/workspaces/${workspace}/accounts`);
	if (!res.ok) throw await res.json();
	return res.json();
}

export async function createAccount(
	workspace: string,
	name: string,
	options: { accountType?: string; parentId?: string; currency?: string } = {}
) {
	const body: Record<string, string> = { name };
	if (options.accountType) body.account_type = options.accountType;
	if (options.parentId) body.parent_id = options.parentId;
	if (options.currency) body.currency = options.currency;

	const res = await request('POST', `/workspaces/${workspace}/accounts`, body);
	if (!res.ok) throw await res.json();
	return res.json();
}

export async function getBalance(workspace: string, account: string) {
	const res = await request('GET', `/workspaces/${workspace}/accounts/${account}/balance`);
	if (!res.ok) throw await res.json();
	return res.json();
}

import type {
	PostingInput,
	IncomeStatement,
	BalanceSheet,
	MultiPeriodIncomeStatement,
	MultiPeriodBalanceSheet,
	MultiPeriodRow
} from '$lib/types';
import type { Quarter } from '$lib/period';
import { quarterMonths, quarterPeriodLabel } from '$lib/period';

// ── Entries ─────────────────────────────────────────────────────────────────

export async function listEntries(workspace: string) {
	const res = await request('GET', `/workspaces/${workspace}/entries`);
	if (!res.ok) throw await res.json();
	return res.json();
}

export async function getEntry(workspace: string, entry: string) {
	const res = await request('GET', `/workspaces/${workspace}/entries/${entry}`);
	if (!res.ok) throw await res.json();
	return res.json();
}

export async function createEntry(
	workspace: string,
	date: string,
	description: string,
	postings: PostingInput[],
	cleared: boolean = false
) {
	const res = await request('POST', `/workspaces/${workspace}/entries`, {
		date,
		description,
		cleared,
		postings
	});
	if (!res.ok) throw await res.json();
	return res.json();
}

export async function deleteEntry(workspace: string, entry: string) {
	const res = await request('DELETE', `/workspaces/${workspace}/entries/${entry}`);
	if (!res.ok) throw await res.json();
	return res.json();
}

// ── Reports ─────────────────────────────────────────────────────────────────

export async function getBalanceSheet(workspace: string, asOf: string) {
	const res = await request(
		'GET',
		`/workspaces/${workspace}/reports/balance-sheet?as_of=${asOf}`
	);
	if (!res.ok) throw await res.json();
	return res.json();
}

export async function getIncomeStatement(
	workspace: string,
	startDate: string,
	endDate: string
) {
	const res = await request(
		'GET',
		`/workspaces/${workspace}/reports/income-statement?start_date=${startDate}&end_date=${endDate}`
	);
	if (!res.ok) throw await res.json();
	return res.json();
}

// ── Period-based Reports ───────────────────────────────────────────────

export async function getPeriodIncomeStatement(
	workspace: string,
	year: number,
	period: string
): Promise<IncomeStatement> {
	const res = await request('GET', `/workspaces/${workspace}/${year}/${period}/income-statement`);
	if (!res.ok) throw await res.json();
	return res.json();
}

export async function getPeriodBalanceSheet(
	workspace: string,
	year: number,
	period: string
): Promise<BalanceSheet> {
	const res = await request('GET', `/workspaces/${workspace}/${year}/${period}/balance-sheet`);
	if (!res.ok) throw await res.json();
	return res.json();
}

export async function getPeriodEntries(workspace: string, year: number, period: string) {
	const res = await request('GET', `/workspaces/${workspace}/${year}/${period}/entries`);
	if (!res.ok) throw await res.json();
	return res.json();
}

// ── Multi-Period Aggregation ───────────────────────────────────────────

function pivotAccountBalances(
	statements: { items: { account: string; name: string; balance: number }[] }[]
): MultiPeriodRow[] {
	const accountMap = new Map<string, { name: string; balances: number[] }>();

	for (let i = 0; i < statements.length; i++) {
		for (const item of statements[i].items) {
			let row = accountMap.get(item.account);
			if (!row) {
				row = { name: item.name, balances: new Array(statements.length).fill(0) };
				accountMap.set(item.account, row);
			}
			row.balances[i] = item.balance;
		}
	}

	return Array.from(accountMap.entries()).map(([account, { name, balances }]) => ({
		account,
		name,
		balances
	}));
}

export async function getMultiPeriodIncomeStatement(
	workspace: string,
	quarter: Quarter
): Promise<MultiPeriodIncomeStatement> {
	const months = quarterMonths(quarter);
	const periodLabel = quarterPeriodLabel(quarter);

	const results = await Promise.all([
		...months.map((month) => getPeriodIncomeStatement(workspace, quarter.year, month)),
		getPeriodIncomeStatement(workspace, quarter.year, periodLabel)
	]);

	const periods = [...months, periodLabel];

	const revenue = pivotAccountBalances(
		results.map((r) => ({ items: r.revenue }))
	);
	const expenses = pivotAccountBalances(
		results.map((r) => ({ items: r.expenses }))
	);

	return {
		periods,
		revenue,
		expenses,
		total_revenue: results.map((r) => r.total_revenue),
		total_expenses: results.map((r) => r.total_expenses),
		net_income: results.map((r) => r.net_income)
	};
}

export async function getMultiPeriodBalanceSheet(
	workspace: string,
	quarter: Quarter
): Promise<MultiPeriodBalanceSheet> {
	const months = quarterMonths(quarter);
	const periodLabel = quarterPeriodLabel(quarter);

	const results = await Promise.all([
		...months.map((month) => getPeriodBalanceSheet(workspace, quarter.year, month)),
		getPeriodBalanceSheet(workspace, quarter.year, periodLabel)
	]);

	const periods = [...months, periodLabel];

	const assets = pivotAccountBalances(results.map((r) => ({ items: r.assets })));
	const liabilities = pivotAccountBalances(results.map((r) => ({ items: r.liabilities })));
	const equity = pivotAccountBalances(results.map((r) => ({ items: r.equity })));

	return {
		periods,
		assets,
		liabilities,
		equity,
		total_assets: results.map((r) => r.total_assets),
		total_liabilities: results.map((r) => r.total_liabilities),
		total_equity: results.map((r) => r.total_equity),
		net_income: results.map((r) => r.net_income),
		total_liabilities_and_equity: results.map((r) => r.total_liabilities_and_equity)
	};
}

// ── Baselines ───────────────────────────────────────────────────────────────

export async function listBaselines(workspace: string) {
	const res = await request('GET', `/workspaces/${workspace}/baselines`);
	if (!res.ok) throw await res.json();
	return res.json();
}

export async function createBaseline(
	workspace: string,
	accountId: string,
	periodStart: string,
	amount: number
) {
	const res = await request('POST', `/workspaces/${workspace}/baselines`, {
		account_id: accountId,
		period_start: periodStart,
		amount
	});
	if (!res.ok) throw await res.json();
	return res.json();
}

export async function deleteBaseline(workspace: string, baseline: string) {
	const res = await request('DELETE', `/workspaces/${workspace}/baselines/${baseline}`);
	if (!res.ok) throw await res.json();
	return res.json();
}

export async function getVariance(workspace: string, baseline: string) {
	const res = await request(
		'GET',
		`/workspaces/${workspace}/baselines/${baseline}/variance`
	);
	if (!res.ok) throw await res.json();
	return res.json();
}
