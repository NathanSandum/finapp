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

// ── Entries ─────────────────────────────────────────────────────────────────

export type PostingInput = {
	account_id: string;
	amount: number;
	note?: string;
};

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
