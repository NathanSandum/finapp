import type { Account, AccountRole, Anchors } from '$lib/views/entries/loader';

export type AccountNode = {
	account: Account;
	children: AccountNode[];
};

export type RootKey = 'assets' | 'liabilities' | 'equity' | 'revenue' | 'expenses';

export const ROOT_ORDER: RootKey[] = [
	'assets',
	'liabilities',
	'equity',
	'revenue',
	'expenses'
];

export const ROOT_LABEL: Record<RootKey, string> = {
	assets: 'Assets',
	liabilities: 'Liabilities',
	equity: 'Equity',
	revenue: 'Revenue',
	expenses: 'Expenses'
};

export const ROOT_ACCENT: Record<RootKey, string> = {
	assets: 'text-emerald-700',
	liabilities: 'text-rose-700',
	equity: 'text-indigo-700',
	revenue: 'text-sky-700',
	expenses: 'text-amber-700'
};

export const ROLE_BADGE: Partial<Record<AccountRole, string>> = {
	assets_root: 'root',
	liabilities_root: 'root',
	equity_root: 'root',
	revenue_root: 'root',
	expenses_root: 'root',
	cash: 'cash',
	opening_equity: 'opening'
};

export function buildSubtree(rootId: string, all: Account[]): AccountNode {
	const childrenByParent = new Map<string, Account[]>();
	for (const a of all) {
		if (!a.parent_id) continue;
		const list = childrenByParent.get(a.parent_id) ?? [];
		list.push(a);
		childrenByParent.set(a.parent_id, list);
	}
	for (const list of childrenByParent.values()) {
		list.sort((a, b) => a.name.localeCompare(b.name));
	}
	const root = all.find((a) => a.id === rootId);
	if (!root) throw new Error(`root ${rootId} not found`);

	function build(account: Account): AccountNode {
		const kids = childrenByParent.get(account.id) ?? [];
		return { account, children: kids.map(build) };
	}
	return build(root);
}

export function countDescendants(node: AccountNode): number {
	let n = node.children.length;
	for (const c of node.children) n += countDescendants(c);
	return n;
}

export function rootIdFor(anchors: Anchors, key: RootKey): string {
	return anchors[key].id;
}
