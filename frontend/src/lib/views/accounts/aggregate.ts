import type { Entry } from '$lib/views/entries/loader';
import { collectIds, type AccountNode } from './tree';

const CUMULATIVE_TYPES = new Set(['asset', 'liability', 'equity']);
const NEGATED_TYPES = new Set(['expense', 'revenue', 'equity']);

export type RecentEntry = {
	id: string;
	date: string;
	description: string;
	amount: number;
};

export function displaySignOf(accountType: string): -1 | 1 {
	return NEGATED_TYPES.has(accountType) ? -1 : 1;
}

export function periodValueOf(
	selected: AccountNode,
	entries: Entry[],
	bounds: { start: string; end: string },
	sign: -1 | 1
): number {
	const ids = new Set<string>();
	collectIds(selected, ids);
	const isCumulative = CUMULATIVE_TYPES.has(selected.account.account_type);
	let amount = 0;
	for (const entry of entries) {
		if (isCumulative) {
			if (entry.date > bounds.end) continue;
		} else {
			if (entry.date < bounds.start || entry.date > bounds.end) continue;
		}
		for (const posting of entry.postings) {
			if (ids.has(posting.account_id)) amount += posting.amount;
		}
	}
	return amount * sign;
}

export function recentEntriesOf(
	selected: AccountNode,
	entries: Entry[],
	sign: -1 | 1,
	limit: number
): RecentEntry[] {
	const ids = new Set<string>();
	collectIds(selected, ids);
	return entries
		.map((entry) => {
			let amount = 0;
			let touched = false;
			for (const posting of entry.postings) {
				if (ids.has(posting.account_id)) {
					amount += posting.amount;
					touched = true;
				}
			}
			return touched
				? {
						id: entry.id,
						date: entry.date,
						description: entry.description,
						amount: amount * sign
					}
				: null;
		})
		.filter((entry): entry is RecentEntry => entry !== null)
		.sort((a, b) => (a.date < b.date ? 1 : a.date > b.date ? -1 : 0))
		.slice(0, limit);
}
