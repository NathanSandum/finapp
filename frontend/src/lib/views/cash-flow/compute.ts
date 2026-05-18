import type { Account, Anchors, Entry } from '../entries/loader';
import {
	asOfDate,
	defaultTrailCount,
	latestActivityPeriod,
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

export { defaultTrailCount, latestActivityPeriod, priorPeriod, shiftPeriod, snapToGranularity };
export type { Granularity };

export type CashFlowLine = { name: string; amount: number };

export type CashFlowSection = {
	lines: CashFlowLine[];
	total: number;
};

export type CashFlowData = {
	period_label: string;
	prior_label: string;
	prior_short_label: string;
	operating: CashFlowSection;
	investing: CashFlowSection;
	financing: CashFlowSection;
	prior_operating_total: number;
	prior_investing_total: number;
	prior_financing_total: number;
	opening_cash: number;
	closing_cash: number;
};

function topBucket(accountId: string, accounts: Map<string, Account>): Account | null {
	let current = accounts.get(accountId);
	if (!current) return null;
	while (current.parent_id) {
		const parent = accounts.get(current.parent_id);
		if (!parent) return null;
		if (!parent.parent_id) return current;
		current = parent;
	}
	return null;
}

type Category = 'operating' | 'investing' | 'financing' | 'cash' | null;

function categorize(bucket: Account, cashAnchorId: string): Category {
	if (bucket.account_type === 'revenue' || bucket.account_type === 'expense') return 'operating';
	if (bucket.account_type === 'asset') return bucket.id === cashAnchorId ? 'cash' : 'investing';
	if (bucket.account_type === 'liability' || bucket.account_type === 'equity') return 'financing';
	return null;
}

function buildSections(
	entries: Entry[],
	accounts: Map<string, Account>,
	cashAnchorId: string,
	start: string,
	end: string
): { operating: CashFlowSection; investing: CashFlowSection; financing: CashFlowSection } {
	const operatingSums = new Map<string, { name: string; amount: number }>();
	const investingSums = new Map<string, { name: string; amount: number }>();
	const financingSums = new Map<string, { name: string; amount: number }>();

	for (const entry of entries) {
		if (entry.date < start || entry.date > end) continue;
		for (const posting of entry.postings) {
			const bucket = topBucket(posting.account_id, accounts);
			if (!bucket) continue;
			const category = categorize(bucket, cashAnchorId);
			if (!category || category === 'cash') continue;

			const cashEffect = -posting.amount;
			const map =
				category === 'operating'
					? operatingSums
					: category === 'investing'
						? investingSums
						: financingSums;
			const existing = map.get(bucket.id);
			if (existing) existing.amount += cashEffect;
			else map.set(bucket.id, { name: bucket.name, amount: cashEffect });
		}
	}

	const toSection = (sums: Map<string, { name: string; amount: number }>): CashFlowSection => {
		const lines: CashFlowLine[] = [];
		let total = 0;
		for (const { name, amount } of sums.values()) {
			total += amount;
			if (amount !== 0) lines.push({ name, amount });
		}
		lines.sort((a, b) => Math.abs(b.amount) - Math.abs(a.amount));
		return { lines, total };
	};

	return {
		operating: toSection(operatingSums),
		investing: toSection(investingSums),
		financing: toSection(financingSums)
	};
}

function cashBalanceAt(
	entries: Entry[],
	accounts: Map<string, Account>,
	cashAnchorId: string,
	asOf: string
): number {
	let total = 0;
	for (const entry of entries) {
		if (entry.date > asOf) continue;
		for (const posting of entry.postings) {
			const bucket = topBucket(posting.account_id, accounts);
			if (!bucket) continue;
			if (bucket.id === cashAnchorId) total += posting.amount;
		}
	}
	return total;
}

export function computeCashFlow(
	entries: Entry[],
	accounts: Account[],
	anchors: Anchors,
	year: number,
	month: number,
	granularity: Granularity
): CashFlowData {
	const accountIndex = new Map(accounts.map((a) => [a.id, a]));
	const cashAnchorId = anchors.cash.id;
	const { start, end } = periodBounds(year, month, granularity);
	const prior = priorPeriod(year, month, granularity);
	const { start: priorStart, end: priorEnd } = periodBounds(prior.year, prior.month, granularity);

	const current = buildSections(entries, accountIndex, cashAnchorId, start, end);
	const previous = buildSections(entries, accountIndex, cashAnchorId, priorStart, priorEnd);

	const opening_cash = cashBalanceAt(entries, accountIndex, cashAnchorId, priorEnd);
	const closing_cash = cashBalanceAt(entries, accountIndex, cashAnchorId, end);

	return {
		period_label: periodLabel(year, month, granularity),
		prior_label: periodLabel(prior.year, prior.month, granularity),
		prior_short_label: priorShortLabel(year, month, granularity),
		operating: current.operating,
		investing: current.investing,
		financing: current.financing,
		prior_operating_total: previous.operating.total,
		prior_investing_total: previous.investing.total,
		prior_financing_total: previous.financing.total,
		opening_cash,
		closing_cash
	};
}

export function computeNetCashTrail(
	entries: Entry[],
	accounts: Account[],
	anchors: Anchors,
	year: number,
	month: number,
	granularity: Granularity,
	count: number
): { values: number[]; labels: string[]; keys: string[] } {
	const accountIndex = new Map(accounts.map((a) => [a.id, a]));
	const cashAnchorId = anchors.cash.id;
	const periods = priorPeriodList(year, month, granularity, count);
	const values: number[] = [];
	const labels: string[] = [];
	const keys: string[] = [];
	for (const period of periods) {
		const { start, end } = periodBounds(period.year, period.month, granularity);
		const sections = buildSections(entries, accountIndex, cashAnchorId, start, end);
		values.push(sections.operating.total + sections.investing.total + sections.financing.total);
		labels.push(trailLabel(period.year, period.month, granularity));
		keys.push(trailKey(period.year, period.month, granularity));
	}
	return { values, labels, keys };
}

export function computeSectionTrails(
	entries: Entry[],
	accounts: Account[],
	anchors: Anchors,
	year: number,
	month: number,
	granularity: Granularity,
	count: number
): {
	operating: number[];
	investing: number[];
	financing: number[];
	labels: string[];
	keys: string[];
} {
	const accountIndex = new Map(accounts.map((a) => [a.id, a]));
	const cashAnchorId = anchors.cash.id;
	const periods = priorPeriodList(year, month, granularity, count);
	const operating: number[] = [];
	const investing: number[] = [];
	const financing: number[] = [];
	const labels: string[] = [];
	const keys: string[] = [];
	for (const period of periods) {
		const { start, end } = periodBounds(period.year, period.month, granularity);
		const sections = buildSections(entries, accountIndex, cashAnchorId, start, end);
		operating.push(sections.operating.total);
		investing.push(sections.investing.total);
		financing.push(sections.financing.total);
		labels.push(trailLabel(period.year, period.month, granularity));
		keys.push(trailKey(period.year, period.month, granularity));
	}
	return { operating, investing, financing, labels, keys };
}

export { asOfDate };
