export type Quarter = {
	year: number;
	quarter: 1 | 2 | 3 | 4;
};

const MONTH_LABELS = [
	'Jan',
	'Feb',
	'Mar',
	'Apr',
	'May',
	'Jun',
	'Jul',
	'Aug',
	'Sep',
	'Oct',
	'Nov',
	'Dec'
] as const;

export function currentQuarter(): Quarter {
	const now = new Date();
	const quarter = (Math.floor(now.getMonth() / 3) + 1) as 1 | 2 | 3 | 4;
	return { year: now.getFullYear(), quarter };
}

export function quarterMonths(quarter: Quarter): string[] {
	const firstMonth = (quarter.quarter - 1) * 3;
	return [
		MONTH_LABELS[firstMonth],
		MONTH_LABELS[firstMonth + 1],
		MONTH_LABELS[firstMonth + 2]
	];
}

export function quarterLabel(quarter: Quarter): string {
	return `Q${quarter.quarter} ${quarter.year}`;
}

export function quarterPeriodLabel(quarter: Quarter): string {
	return `Q${quarter.quarter}`;
}

export function previousQuarter(quarter: Quarter): Quarter {
	if (quarter.quarter === 1) {
		return { year: quarter.year - 1, quarter: 4 };
	}
	return { year: quarter.year, quarter: (quarter.quarter - 1) as 1 | 2 | 3 | 4 };
}

export function nextQuarter(quarter: Quarter): Quarter {
	if (quarter.quarter === 4) {
		return { year: quarter.year + 1, quarter: 1 };
	}
	return { year: quarter.year, quarter: (quarter.quarter + 1) as 1 | 2 | 3 | 4 };
}

export function currentMonthPeriod(): { periodStart: string; periodEnd: string } {
	const now = new Date();
	const periodStart = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}-01`;
	const lastDay = new Date(now.getFullYear(), now.getMonth() + 1, 0);
	const periodEnd = `${lastDay.getFullYear()}-${String(lastDay.getMonth() + 1).padStart(2, '0')}-${String(lastDay.getDate()).padStart(2, '0')}`;
	return { periodStart, periodEnd };
}
