export type Granularity = 'month' | 'quarter' | 'year';

const MONTH_LONG = [
	'January', 'February', 'March', 'April', 'May', 'June',
	'July', 'August', 'September', 'October', 'November', 'December'
];
const MONTH_SHORT = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];

function pad2(n: number): string {
	return n.toString().padStart(2, '0');
}

export function quarterOf(month: number): number {
	return Math.ceil(month / 3);
}

function quarterStartMonth(quarter: number): number {
	return (quarter - 1) * 3 + 1;
}

function lastDayOfMonth(year: number, month: number): number {
	return new Date(Date.UTC(year, month, 0)).getUTCDate();
}

export function periodBounds(
	year: number,
	month: number,
	granularity: Granularity
): { start: string; end: string } {
	if (granularity === 'month') {
		return {
			start: `${year}-${pad2(month)}-01`,
			end: `${year}-${pad2(month)}-${pad2(lastDayOfMonth(year, month))}`
		};
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

export function asOfDate(year: number, month: number, granularity: Granularity): string {
	return periodBounds(year, month, granularity).end;
}

export function priorPeriod(
	year: number,
	month: number,
	granularity: Granularity
): { year: number; month: number } {
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

export function snapToGranularity(
	year: number,
	month: number,
	granularity: Granularity
): { year: number; month: number } {
	if (granularity === 'month') return { year, month };
	if (granularity === 'quarter') return { year, month: quarterStartMonth(quarterOf(month)) };
	return { year, month: 1 };
}

export function defaultTrailCount(granularity: Granularity): number {
	if (granularity === 'month') return 12;
	if (granularity === 'quarter') return 8;
	return 5;
}

export function periodLabel(year: number, month: number, granularity: Granularity): string {
	if (granularity === 'month') return `${MONTH_LONG[month - 1]} ${year}`;
	if (granularity === 'quarter') return `Q${quarterOf(month)} ${year}`;
	return `${year}`;
}

export function asOfLabel(year: number, month: number, granularity: Granularity): string {
	if (granularity === 'month') return `${MONTH_LONG[month - 1]} ${lastDayOfMonth(year, month)}, ${year}`;
	if (granularity === 'quarter') return `Q${quarterOf(month)} ${year}`;
	return `${year}`;
}

export function priorShortLabel(year: number, month: number, granularity: Granularity): string {
	const prior = priorPeriod(year, month, granularity);
	if (granularity === 'month') return MONTH_SHORT[prior.month - 1];
	if (granularity === 'quarter') return `Q${quarterOf(prior.month)}`;
	return `${prior.year}`;
}

export function trailLabel(year: number, month: number, granularity: Granularity): string {
	if (granularity === 'month') return MONTH_SHORT[month - 1];
	if (granularity === 'quarter') return `Q${quarterOf(month)}`;
	return `'${(year % 100).toString().padStart(2, '0')}`;
}

export function latestActivityPeriod(
	entries: { date: string }[]
): { year: number; month: number } {
	let latest: string | null = null;
	for (const entry of entries) {
		if (latest === null || entry.date > latest) latest = entry.date;
	}
	if (latest) {
		const [y, m] = latest.split('-');
		return { year: Number(y), month: Number(m) };
	}
	const now = new Date();
	return { year: now.getUTCFullYear(), month: now.getUTCMonth() + 1 };
}

export function priorPeriodList(
	year: number,
	month: number,
	granularity: Granularity,
	count: number
): { year: number; month: number }[] {
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
	return periods;
}
