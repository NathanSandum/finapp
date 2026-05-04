import type { Account, Entry } from '../entries/loader';
import type { IncomeData, Line } from './lib';

export type Granularity = 'month' | 'quarter' | 'year';

const MONTH_LONG = [
	'January', 'February', 'March', 'April', 'May', 'June',
	'July', 'August', 'September', 'October', 'November', 'December'
];
const MONTH_SHORT = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];

function pad2(n: number): string {
	return n.toString().padStart(2, '0');
}

function quarterOf(month: number): number {
	return Math.ceil(month / 3);
}

function quarterStartMonth(quarter: number): number {
	return (quarter - 1) * 3 + 1;
}

function lastDayOfMonth(year: number, month: number): number {
	return new Date(Date.UTC(year, month, 0)).getUTCDate();
}

function bounds(year: number, month: number, granularity: Granularity): { start: string; end: string } {
	if (granularity === 'month') {
		return { start: `${year}-${pad2(month)}-01`, end: `${year}-${pad2(month)}-${pad2(lastDayOfMonth(year, month))}` };
	}
	if (granularity === 'quarter') {
		const q = quarterOf(month);
		const startM = quarterStartMonth(q);
		const endM = startM + 2;
		return {
			start: `${year}-${pad2(startM)}-01`,
			end: `${year}-${pad2(endM)}-${pad2(lastDayOfMonth(year, endM))}`
		};
	}
	return { start: `${year}-01-01`, end: `${year}-12-31` };
}

export function priorPeriod(year: number, month: number, granularity: Granularity): { year: number; month: number } {
	if (granularity === 'month') {
		if (month === 1) return { year: year - 1, month: 12 };
		return { year, month: month - 1 };
	}
	if (granularity === 'quarter') {
		const q = quarterOf(month);
		if (q === 1) return { year: year - 1, month: quarterStartMonth(4) };
		return { year, month: quarterStartMonth(q - 1) };
	}
	return { year: year - 1, month: 1 };
}

export function shiftPeriod(
	year: number,
	month: number,
	granularity: Granularity,
	direction: 1 | -1
): { year: number; month: number } {
	if (granularity === 'month') {
		let m = month + direction;
		let y = year;
		while (m < 1) {
			m += 12;
			y -= 1;
		}
		while (m > 12) {
			m -= 12;
			y += 1;
		}
		return { year: y, month: m };
	}
	if (granularity === 'quarter') {
		const q = quarterOf(month);
		let nq = q + direction;
		let y = year;
		while (nq < 1) {
			nq += 4;
			y -= 1;
		}
		while (nq > 4) {
			nq -= 4;
			y += 1;
		}
		return { year: y, month: quarterStartMonth(nq) };
	}
	return { year: year + direction, month: 1 };
}

export function snapToGranularity(year: number, month: number, granularity: Granularity): { year: number; month: number } {
	if (granularity === 'month') return { year, month };
	if (granularity === 'quarter') return { year, month: quarterStartMonth(quarterOf(month)) };
	return { year, month: 1 };
}

function periodLabel(year: number, month: number, granularity: Granularity): string {
	if (granularity === 'month') return `${MONTH_LONG[month - 1]} ${year}`;
	if (granularity === 'quarter') return `Q${quarterOf(month)} ${year}`;
	return `${year}`;
}

function priorShort(year: number, month: number, granularity: Granularity): string {
	const prior = priorPeriod(year, month, granularity);
	if (granularity === 'month') return MONTH_SHORT[prior.month - 1];
	if (granularity === 'quarter') return `Q${quarterOf(prior.month)}`;
	return `${prior.year}`;
}

function trailLabel(year: number, month: number, granularity: Granularity): string {
	if (granularity === 'month') return MONTH_SHORT[month - 1];
	if (granularity === 'quarter') return `Q${quarterOf(month)}`;
	return `'${(year % 100).toString().padStart(2, '0')}`;
}

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
	const { start, end } = bounds(year, month, granularity);
	const prior = priorPeriod(year, month, granularity);
	const { start: priorStart, end: priorEnd } = bounds(prior.year, prior.month, granularity);

	const current = buildLines(entries, accountIndex, start, end);
	const priorBucket = buildLines(entries, accountIndex, priorStart, priorEnd);

	return {
		period_label: periodLabel(year, month, granularity),
		prior_label: periodLabel(prior.year, prior.month, granularity),
		prior_short_label: priorShort(year, month, granularity),
		revenue: current.revenue,
		expenses: current.expenses,
		prior_revenue: priorBucket.revenue,
		prior_expenses: priorBucket.expenses
	};
}

export function defaultTrailCount(granularity: Granularity): number {
	if (granularity === 'month') return 12;
	if (granularity === 'quarter') return 8;
	return 5;
}

export function computeNetTrail(
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
		const { start, end } = bounds(period.year, period.month, granularity);
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
	}
	return { values, labels };
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
	const now = new Date();
	return { year: now.getUTCFullYear(), month: now.getUTCMonth() + 1 };
}
