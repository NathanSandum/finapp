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

export async function listAccounts(workspace: string) {
	const res = await request('GET', `/workspaces/${workspace}/accounts`);
	if (!res.ok) throw await res.json();
	return res.json();
}

export async function listTransactions(workspace: string, account: string) {
	const res = await request('GET', `/workspaces/${workspace}/accounts/${account}/transactions`);
	if (!res.ok) throw await res.json();
	return res.json();
}

export async function getBalance(workspace: string, account: string) {
	const res = await request('GET', `/workspaces/${workspace}/accounts/${account}/balance`);
	if (!res.ok) throw await res.json();
	return res.json();
}

export async function createAccount(
	workspace: string,
	name: string,
	accountType: string
) {
	const res = await request('POST', `/workspaces/${workspace}/accounts`, {
		name,
		account_type: accountType
	});
	if (!res.ok) throw await res.json();
	return res.json();
}

export async function listCategories(workspace: string) {
	const res = await request('GET', `/workspaces/${workspace}/categories`);
	if (!res.ok) throw await res.json();
	return res.json();
}

export async function createCategory(workspace: string, name: string, description?: string) {
	const res = await request('POST', `/workspaces/${workspace}/categories`, {
		name,
		description
	});
	if (!res.ok) throw await res.json();
	return res.json();
}

export async function listBaselines(workspace: string, periodStart: string) {
	const res = await request(
		'GET',
		`/workspaces/${workspace}/baselines?period_start=${periodStart}`
	);
	if (!res.ok) throw await res.json();
	return res.json();
}

export async function baselineSummary(
	workspace: string,
	periodStart: string,
	periodEnd: string
) {
	const res = await request(
		'GET',
		`/workspaces/${workspace}/baselines/summary?period_start=${periodStart}&period_end=${periodEnd}`
	);
	if (!res.ok) throw await res.json();
	return res.json();
}

export async function createTransaction(
	workspace: string,
	account: string,
	date: string,
	amount: number,
	lineItems: { category_id: string; amount: number }[],
	description?: string,
	cleared?: boolean
) {
	const res = await request('POST', `/workspaces/${workspace}/transactions`, {
		account,
		date,
		amount,
		line_items: lineItems,
		description,
		cleared
	});
	if (!res.ok) throw await res.json();
	return res.json();
}
