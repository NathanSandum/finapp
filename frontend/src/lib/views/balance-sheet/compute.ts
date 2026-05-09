import type { Account, Entry } from '../entries/loader';
import {
	asOfDate,
	asOfLabel,
	defaultTrailCount,
	priorPeriod,
	priorShortLabel,
	shiftPeriod,
	snapToGranularity,
	trailLabel,
	type Granularity
} from '../period';

export { defaultTrailCount, priorPeriod, shiftPeriod, snapToGranularity };
export type { Granularity };

export type Line = { name: string; amount: number };

export type BalanceSheetData = {
	as_of_label: string;
	prior_label: string;
	prior_short_label: string;
	assets: Line[];
	liabilities: Line[];
	prior_assets: Line[];
	prior_liabilities: Line[];
};

function bucketFor(accountId: string, accounts: Map<string, Account>): Account | null {
	let current = accounts.get(accountId);
	if (!current) return null;
	if (current.account_type !== 'asset' && current.account_type !== 'liability') return null;

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
	asOf: string
): { assets: Line[]; liabilities: Line[] } {
	const sums = new Map<string, { account: Account; amount: number }>();

	for (const entry of entries) {
		if (entry.date > asOf) continue;
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

	const assets: Line[] = [];
	const liabilities: Line[] = [];
	for (const { account, amount } of sums.values()) {
		const display = account.account_type === 'asset' ? amount : -amount;
		if (display === 0) continue;
		const line: Line = { name: account.name, amount: display };
		if (account.account_type === 'asset') assets.push(line);
		else liabilities.push(line);
	}
	assets.sort((a, b) => b.amount - a.amount);
	liabilities.sort((a, b) => b.amount - a.amount);
	return { assets, liabilities };
}

export function computeBalanceSheet(
	entries: Entry[],
	accounts: Account[],
	year: number,
	month: number,
	granularity: Granularity
): BalanceSheetData {
	const accountIndex = new Map(accounts.map((a) => [a.id, a]));
	const asOf = asOfDate(year, month, granularity);
	const prior = priorPeriod(year, month, granularity);
	const priorAsOf = asOfDate(prior.year, prior.month, granularity);

	const current = buildLines(entries, accountIndex, asOf);
	const previous = buildLines(entries, accountIndex, priorAsOf);

	return {
		as_of_label: asOfLabel(year, month, granularity),
		prior_label: asOfLabel(prior.year, prior.month, granularity),
		prior_short_label: priorShortLabel(year, month, granularity),
		assets: current.assets,
		liabilities: current.liabilities,
		prior_assets: previous.assets,
		prior_liabilities: previous.liabilities
	};
}

export function computeNetWorthTrail(
	entries: Entry[],
	accounts: Account[],
	year: number,
	month: number,
	granularity: Granularity,
	count: number
): { values: number[]; labels: string[] } {
	const accountIndex = new Map(accounts.map((a) => [a.id, a]));
	const periods: { year: number; month: number }[] = [];
	let y = year;
	let m = month;
	for (let i = 0; i < count; i++) {
		periods.push({ year: y, month: m });
		const p = priorPeriod(y, m, granularity);
		y = p.year;
		m = p.month;
	}
	periods.reverse();

	const values: number[] = [];
	const labels: string[] = [];
	for (const period of periods) {
		const asOf = asOfDate(period.year, period.month, granularity);
		let assets = 0;
		let liabilities = 0;
		for (const entry of entries) {
			if (entry.date > asOf) continue;
			for (const posting of entry.postings) {
				const bucket = bucketFor(posting.account_id, accountIndex);
				if (!bucket) continue;
				if (bucket.account_type === 'asset') assets += posting.amount;
				else if (bucket.account_type === 'liability') liabilities += -posting.amount;
			}
		}
		values.push(assets - liabilities);
		labels.push(trailLabel(period.year, period.month, granularity));
	}
	return { values, labels };
}

export function computeAssetTrails(
	entries: Entry[],
	accounts: Account[],
	year: number,
	month: number,
	granularity: Granularity,
	count: number
): { trails: Map<string, number[]>; labels: string[] } {
	const accountIndex = new Map(accounts.map((a) => [a.id, a]));
	const periods: { year: number; month: number }[] = [];
	let y = year;
	let m = month;
	for (let i = 0; i < count; i++) {
		periods.push({ year: y, month: m });
		const p = priorPeriod(y, m, granularity);
		y = p.year;
		m = p.month;
	}
	periods.reverse();

	const trails = new Map<string, number[]>();
	const labels: string[] = [];
	for (let pi = 0; pi < periods.length; pi++) {
		const period = periods[pi];
		const asOf = asOfDate(period.year, period.month, granularity);
		const periodSums = new Map<string, number>();
		for (const entry of entries) {
			if (entry.date > asOf) continue;
			for (const posting of entry.postings) {
				const bucket = bucketFor(posting.account_id, accountIndex);
				if (!bucket || bucket.account_type !== 'asset') continue;
				periodSums.set(bucket.name, (periodSums.get(bucket.name) ?? 0) + posting.amount);
			}
		}
		for (const [name, sum] of periodSums) {
			let arr = trails.get(name);
			if (!arr) {
				arr = new Array(periods.length).fill(0);
				trails.set(name, arr);
			}
			arr[pi] = sum;
		}
		labels.push(trailLabel(period.year, period.month, granularity));
	}
	return { trails, labels };
}

export function latestActivityPeriod(
	entries: Entry[],
	accounts: Account[]
): { year: number; month: number } {
	const accountIndex = new Map(accounts.map((a) => [a.id, a]));
	let latest: string | null = null;
	for (const entry of entries) {
		for (const posting of entry.postings) {
			const bucket = bucketFor(posting.account_id, accountIndex);
			if (!bucket) continue;
			if (latest === null || entry.date > latest) latest = entry.date;
			break;
		}
	}
	if (latest) {
		const [y, m] = latest.split('-');
		return { year: Number(y), month: Number(m) };
	}
	const now = new Date();
	return { year: now.getUTCFullYear(), month: now.getUTCMonth() + 1 };
}
