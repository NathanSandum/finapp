import type { Account, Anchors, Entry } from '../entries/loader';
import { periodBounds, periodLabel, type Granularity } from '../period';

const MONTH_SHORT = [
	'Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun',
	'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'
];

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

function shiftMonth(year: number, month: number, dir: number): { year: number; month: number } {
	let m = month + dir;
	let y = year;
	while (m < 1) { m += 12; y -= 1; }
	while (m > 12) { m -= 12; y += 1; }
	return { year: y, month: m };
}

export type Ratio = {
	value: number;
	display: string;
	health: number;
	target: string;
};

export type OverviewMetrics = {
	asOf: string;
	asOfLabel: string;
	netWorth: number;
	priorNetWorth: number;
	totalAssets: number;
	totalLiabilities: number;
	liquidAssets: number;
	income12mo: number;
	expenses12mo: number;
	priorIncome12mo: number;
	expenses3moAvg: number;
	netWorthTrail: number[];
	netWorthLabels: string[];
	incomeTrail: number[];
	expenseTrail: number[];
	monthLabels: string[];
	emergencyFund: Ratio;
	savingsRate: Ratio;
	debtToIncome: Ratio;
	debtToAssets: Ratio;
};

export function computeOverview(
	entries: Entry[],
	accounts: Account[],
	anchors: Anchors,
	year: number,
	month: number,
	granularity: Granularity
): OverviewMetrics {
	const bounds = periodBounds(year, month, granularity);
	const asOf = bounds.end;

	const accountIndex = new Map(accounts.map((a) => [a.id, a]));
	const cashAnchorId = anchors.cash.id;

	const acctBalance = new Map<string, number>();
	for (const entry of entries) {
		if (entry.date > asOf) continue;
		for (const posting of entry.postings) {
			acctBalance.set(
				posting.account_id,
				(acctBalance.get(posting.account_id) ?? 0) + posting.amount
			);
		}
	}

	let totalAssets = 0;
	let totalLiabilities = 0;
	let liquidAssets = 0;
	for (const [acctId, bal] of acctBalance) {
		const bucket = topBucket(acctId, accountIndex);
		if (!bucket) continue;
		if (bucket.account_type === 'asset') {
			totalAssets += bal;
			if (bucket.id === cashAnchorId) liquidAssets += bal;
		} else if (bucket.account_type === 'liability') {
			totalLiabilities += -bal;
		}
	}
	const netWorth = totalAssets - totalLiabilities;

	const [yStr, mStr] = asOf.split('-');
	const asOfYear = Number(yStr);
	const asOfMonth = Number(mStr);
	const asOfLabel = periodLabel(year, month, granularity);

	let income12mo = 0;
	let expenses12mo = 0;
	let priorIncome12mo = 0;
	const incomeTrail: number[] = [];
	const expenseTrail: number[] = [];
	const monthLabels: string[] = [];
	for (let i = 11; i >= 0; i--) {
		const { year, month } = shiftMonth(asOfYear, asOfMonth, -i);
		const { start, end } = periodBounds(year, month, 'month');
		let rev = 0;
		let exp = 0;
		for (const entry of entries) {
			if (entry.date < start || entry.date > end) continue;
			for (const posting of entry.postings) {
				const bucket = topBucket(posting.account_id, accountIndex);
				if (!bucket) continue;
				if (bucket.account_type === 'revenue') rev += -posting.amount;
				else if (bucket.account_type === 'expense') exp += posting.amount;
			}
		}
		income12mo += rev;
		expenses12mo += exp;
		incomeTrail.push(rev);
		expenseTrail.push(exp);
		monthLabels.push(MONTH_SHORT[month - 1]);
	}

	for (let i = 23; i >= 12; i--) {
		const { year, month } = shiftMonth(asOfYear, asOfMonth, -i);
		const { start, end } = periodBounds(year, month, 'month');
		for (const entry of entries) {
			if (entry.date < start || entry.date > end) continue;
			for (const posting of entry.postings) {
				const bucket = topBucket(posting.account_id, accountIndex);
				if (!bucket) continue;
				if (bucket.account_type === 'revenue') priorIncome12mo += -posting.amount;
			}
		}
	}

	let expenses3moSum = 0;
	for (let i = 2; i >= 0; i--) {
		const { year, month } = shiftMonth(asOfYear, asOfMonth, -i);
		const { start, end } = periodBounds(year, month, 'month');
		for (const entry of entries) {
			if (entry.date < start || entry.date > end) continue;
			for (const posting of entry.postings) {
				const bucket = topBucket(posting.account_id, accountIndex);
				if (!bucket) continue;
				if (bucket.account_type === 'expense') expenses3moSum += posting.amount;
			}
		}
	}
	const expenses3moAvg = expenses3moSum / 3;

	const netWorthTrail: number[] = [];
	const netWorthLabels: string[] = [];
	for (let i = 11; i >= 0; i--) {
		const { year, month } = shiftMonth(asOfYear, asOfMonth, -i);
		const { end } = periodBounds(year, month, 'month');
		let assets = 0;
		let liabilities = 0;
		for (const entry of entries) {
			if (entry.date > end) continue;
			for (const posting of entry.postings) {
				const bucket = topBucket(posting.account_id, accountIndex);
				if (!bucket) continue;
				if (bucket.account_type === 'asset') assets += posting.amount;
				else if (bucket.account_type === 'liability') liabilities += -posting.amount;
			}
		}
		netWorthTrail.push(assets - liabilities);
		netWorthLabels.push(MONTH_SHORT[month - 1]);
	}
	const priorNetWorth = netWorthTrail.length > 1 ? netWorthTrail[netWorthTrail.length - 2] : netWorth;

	const efMonths = expenses3moAvg > 0 ? liquidAssets / expenses3moAvg : 0;
	const srRate = income12mo > 0 ? (income12mo - expenses12mo) / income12mo : 0;
	const di = income12mo > 0 ? totalLiabilities / income12mo : 0;
	const da = totalAssets > 0 ? totalLiabilities / totalAssets : 0;

	const emergencyFund: Ratio = {
		value: efMonths,
		display: `${efMonths.toFixed(1)} mo`,
		health: clamp01(efMonths / 6),
		target: '≥ 6 mo'
	};
	const savingsRate: Ratio = {
		value: srRate,
		display: `${(srRate * 100).toFixed(1)}%`,
		health: clamp01(srRate / 0.2),
		target: '≥ 20%'
	};
	const debtToIncome: Ratio = {
		value: di,
		display: `${(di * 100).toFixed(1)}%`,
		health: clamp01(1 - di / 2),
		target: '≤ 36%'
	};
	const debtToAssets: Ratio = {
		value: da,
		display: `${(da * 100).toFixed(1)}%`,
		health: clamp01(1 - da),
		target: '≤ 50%'
	};

	return {
		asOf,
		asOfLabel,
		netWorth,
		priorNetWorth,
		totalAssets,
		totalLiabilities,
		liquidAssets,
		income12mo,
		expenses12mo,
		priorIncome12mo,
		expenses3moAvg,
		netWorthTrail,
		netWorthLabels,
		incomeTrail,
		expenseTrail,
		monthLabels,
		emergencyFund,
		savingsRate,
		debtToIncome,
		debtToAssets
	};
}

function clamp01(n: number): number {
	if (Number.isNaN(n)) return 0;
	return Math.max(0, Math.min(1, n));
}
