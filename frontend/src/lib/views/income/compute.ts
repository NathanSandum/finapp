import type { Account, Entry } from '$lib/data';
import {
	defaultTrailCount,
	periodBounds,
	periodLabel,
	priorPeriod,
	priorPeriodList,
	priorShortLabel,
	shiftPeriod,
	snapToGranularity,
	trailKey,
	trailLabel,
	type Granularity
} from '../period';
import { todayYearMonth } from '../today';
import type { IncomeData, Line } from './lib';

export { defaultTrailCount, priorPeriod, shiftPeriod, snapToGranularity };
export type { Granularity };

function bucketFor(accountId: string, accounts: Map<string, Account>): Account | null {
	let current = accounts.get(accountId);
	if (!current) return null;
	if (current.account_type !== 'revenue' && current.account_type !== 'expense') return null;

	while (current.parent_id) {
		const parent = accounts.get(current.parent_id);
		if (!parent) return null;
		if (!parent.parent_id) return current;
		current = parent;
	}
	return null;
}

function buildLines(
	entries: Entry[],
	accounts: Map<string, Account>,
	start: string,
	end: string
): { revenue: Line[]; expenses: Line[] } {
	const sums = new Map<string, { account: Account; amount: number }>();

	for (const entry of entries) {
		if (entry.date < start || entry.date > end) continue;
		for (const posting of entry.postings) {
			const bucket = bucketFor(posting.account_id, accounts);
			if (!bucket) continue;
			const existing = sums.get(bucket.id);
			if (existing) {
				existing.amount += posting.amount;
			} else {
				sums.set(bucket.id, { account: bucket, amount: posting.amount });
			}
		}
	}

	const revenue: Line[] = [];
	const expenses: Line[] = [];
	for (const { account, amount } of sums.values()) {
		const display = Math.abs(amount);
		if (display === 0) continue;
		const line: Line = { name: account.name, amount: display };
		if (account.account_type === 'revenue') revenue.push(line);
		else if (account.account_type === 'expense') expenses.push(line);
	}
	revenue.sort((a, b) => b.amount - a.amount);
	expenses.sort((a, b) => b.amount - a.amount);
	return { revenue, expenses };
}

export function computeIncomeData(
	entries: Entry[],
	accounts: Account[],
	year: number,
	month: number,
	granularity: Granularity
): IncomeData {
	const accountIndex = new Map(accounts.map((a) => [a.id, a]));
	const { start, end } = periodBounds(year, month, granularity);
	const prior = priorPeriod(year, month, granularity);
	const { start: priorStart, end: priorEnd } = periodBounds(prior.year, prior.month, granularity);

	const current = buildLines(entries, accountIndex, start, end);
	const priorBucket = buildLines(entries, accountIndex, priorStart, priorEnd);

	return {
		period_label: periodLabel(year, month, granularity),
		prior_label: periodLabel(prior.year, prior.month, granularity),
		prior_short_label: priorShortLabel(year, month, granularity),
		revenue: current.revenue,
		expenses: current.expenses,
		prior_revenue: priorBucket.revenue,
		prior_expenses: priorBucket.expenses
	};
}

export function computeNetTrail(
	entries: Entry[],
	accounts: Account[],
	year: number,
	month: number,
	granularity: Granularity,
	count: number
): { values: number[]; labels: string[]; keys: string[] } {
	const accountIndex = new Map(accounts.map((a) => [a.id, a]));
	const periods = priorPeriodList(year, month, granularity, count);

	const values: number[] = [];
	const labels: string[] = [];
	const keys: string[] = [];
	for (const period of periods) {
		const { start, end } = periodBounds(period.year, period.month, granularity);
		let revenue = 0;
		let expense = 0;
		for (const entry of entries) {
			if (entry.date < start || entry.date > end) continue;
			for (const posting of entry.postings) {
				const bucket = bucketFor(posting.account_id, accountIndex);
				if (!bucket) continue;
				if (bucket.account_type === 'revenue') revenue += -posting.amount;
				else if (bucket.account_type === 'expense') expense += posting.amount;
			}
		}
		values.push(revenue - expense);
		labels.push(trailLabel(period.year, period.month, granularity));
		keys.push(trailKey(period.year, period.month, granularity));
	}
	return { values, labels, keys };
}

export function computeIncomeTrails(
	entries: Entry[],
	accounts: Account[],
	year: number,
	month: number,
	granularity: Granularity,
	count: number
): {
	revenueTrails: Map<string, number[]>;
	expenseTrails: Map<string, number[]>;
	labels: string[];
	keys: string[];
} {
	const accountIndex = new Map(accounts.map((a) => [a.id, a]));
	const periods = priorPeriodList(year, month, granularity, count);

	const revenueTrails = new Map<string, number[]>();
	const expenseTrails = new Map<string, number[]>();
	const labels: string[] = [];
	const keys: string[] = [];
	for (let pi = 0; pi < periods.length; pi++) {
		const period = periods[pi];
		const { start, end } = periodBounds(period.year, period.month, granularity);
		const revSums = new Map<string, number>();
		const expSums = new Map<string, number>();
		for (const entry of entries) {
			if (entry.date < start || entry.date > end) continue;
			for (const posting of entry.postings) {
				const bucket = bucketFor(posting.account_id, accountIndex);
				if (!bucket) continue;
				if (bucket.account_type === 'revenue') {
					revSums.set(bucket.name, (revSums.get(bucket.name) ?? 0) + -posting.amount);
				} else if (bucket.account_type === 'expense') {
					expSums.set(bucket.name, (expSums.get(bucket.name) ?? 0) + posting.amount);
				}
			}
		}
		for (const [name, sum] of revSums) {
			let arr = revenueTrails.get(name);
			if (!arr) {
				arr = new Array(periods.length).fill(0);
				revenueTrails.set(name, arr);
			}
			arr[pi] = sum;
		}
		for (const [name, sum] of expSums) {
			let arr = expenseTrails.get(name);
			if (!arr) {
				arr = new Array(periods.length).fill(0);
				expenseTrails.set(name, arr);
			}
			arr[pi] = sum;
		}
		labels.push(trailLabel(period.year, period.month, granularity));
		keys.push(trailKey(period.year, period.month, granularity));
	}
	return { revenueTrails, expenseTrails, labels, keys };
}

export function latestActivityMonth(
	entries: Entry[],
	accounts: Account[]
): { year: number; month: number } {
	const accountIndex = new Map(accounts.map((a) => [a.id, a]));
	let latest: string | null = null;
	for (const entry of entries) {
		for (const posting of entry.postings) {
			const bucket = bucketFor(posting.account_id, accountIndex);
			if (!bucket) continue;
			if (bucket.account_type !== 'revenue' && bucket.account_type !== 'expense') continue;
			if (latest === null || entry.date > latest) latest = entry.date;
			break;
		}
	}
	if (latest) {
		const [y, m] = latest.split('-');
		return { year: Number(y), month: Number(m) };
	}
	return todayYearMonth();
}
