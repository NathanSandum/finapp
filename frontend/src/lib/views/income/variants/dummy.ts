export type Line = { name: string; amount: number; children?: Line[] };

export type IncomeData = {
	period_label: string;
	prior_label: string;
	revenue: Line[];
	expenses: Line[];
	prior_revenue: Line[];
	prior_expenses: Line[];
};

export const dummy: IncomeData = {
	period_label: 'April 2026',
	prior_label: 'March 2026',
	revenue: [
		{
			name: 'Employment',
			amount: 587500,
			children: [
				{ name: 'Salary', amount: 550000 },
				{ name: 'Side gig', amount: 37500 }
			]
		},
		{
			name: 'Other',
			amount: 4200,
			children: [{ name: 'Interest', amount: 4200 }]
		}
	],
	expenses: [
		{
			name: 'Housing',
			amount: 206500,
			children: [
				{ name: 'Rent', amount: 180000 },
				{ name: 'Utilities', amount: 18000 },
				{ name: 'Internet', amount: 8500 }
			]
		},
		{
			name: 'Food',
			amount: 96500,
			children: [
				{ name: 'Groceries', amount: 62000 },
				{ name: 'Restaurants', amount: 21000 },
				{ name: 'Cafes', amount: 13500 }
			]
		},
		{
			name: 'Transport',
			amount: 25500,
			children: [
				{ name: 'Fuel', amount: 21000 },
				{ name: 'Transit', amount: 4500 }
			]
		},
		{ name: 'Healthcare', amount: 5800 },
		{ name: 'Entertainment', amount: 9500 },
		{ name: 'Subscriptions', amount: 4200 }
	],
	prior_revenue: [
		{
			name: 'Employment',
			amount: 582000,
			children: [
				{ name: 'Salary', amount: 550000 },
				{ name: 'Side gig', amount: 32000 }
			]
		},
		{
			name: 'Other',
			amount: 4100,
			children: [{ name: 'Interest', amount: 4100 }]
		}
	],
	prior_expenses: [
		{
			name: 'Housing',
			amount: 210000,
			children: [
				{ name: 'Rent', amount: 180000 },
				{ name: 'Utilities', amount: 21500 },
				{ name: 'Internet', amount: 8500 }
			]
		},
		{
			name: 'Food',
			amount: 85800,
			children: [
				{ name: 'Groceries', amount: 58000 },
				{ name: 'Restaurants', amount: 18000 },
				{ name: 'Cafes', amount: 9800 }
			]
		},
		{
			name: 'Transport',
			amount: 24000,
			children: [
				{ name: 'Fuel', amount: 19500 },
				{ name: 'Transit', amount: 4500 }
			]
		},
		{ name: 'Healthcare', amount: 12000 },
		{ name: 'Entertainment', amount: 6500 },
		{ name: 'Subscriptions', amount: 4200 }
	]
};

export function totalRevenue(data: IncomeData): number {
	return data.revenue.reduce((sum, line) => sum + line.amount, 0);
}

export function totalExpenses(data: IncomeData): number {
	return data.expenses.reduce((sum, line) => sum + line.amount, 0);
}

export function totalPriorRevenue(data: IncomeData): number {
	return data.prior_revenue.reduce((sum, line) => sum + line.amount, 0);
}

export function totalPriorExpenses(data: IncomeData): number {
	return data.prior_expenses.reduce((sum, line) => sum + line.amount, 0);
}

export function fmt(cents: number): string {
	const sign = cents < 0 ? '-' : '';
	const abs = Math.abs(cents);
	const dollars = Math.floor(abs / 100);
	const c = (abs % 100).toString().padStart(2, '0');
	return `${sign}$${dollars.toLocaleString()}.${c}`;
}

export function pct(current: number, prior: number): string {
	if (prior === 0) return current === 0 ? '0%' : '—';
	const change = ((current - prior) / Math.abs(prior)) * 100;
	const sign = change > 0 ? '+' : '';
	return `${sign}${change.toFixed(1)}%`;
}

export function delta(current: number, prior: number): string {
	const diff = current - prior;
	const sign = diff > 0 ? '+' : diff < 0 ? '-' : '';
	const abs = Math.abs(diff);
	const dollars = Math.floor(abs / 100);
	const c = (abs % 100).toString().padStart(2, '0');
	return `${sign}$${dollars.toLocaleString()}.${c}`;
}

export function findPrior(name: string, prior: Line[]): Line | undefined {
	for (const line of prior) {
		if (line.name === name) return line;
		if (line.children) {
			const hit = findPrior(name, line.children);
			if (hit) return hit;
		}
	}
	return undefined;
}

export const months = ['May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec', 'Jan', 'Feb', 'Mar', 'Apr'];

export const priorNetCached = totalPriorRevenue(dummy) - totalPriorExpenses(dummy);
export const netCached = totalRevenue(dummy) - totalExpenses(dummy);

export const netTrail = [
	175000, 195000, 162000, 150000, 200000, 220000, 145000, 80000, 220000, 195000, priorNetCached, netCached
];
export const revTrail = [
	565000, 575000, 580000, 575000, 580000, 600000, 590000, 595000, 600000, 585000,
	totalPriorRevenue(dummy), totalRevenue(dummy)
];
export const expTrail = [
	390000, 380000, 418000, 425000, 380000, 380000, 445000, 515000, 380000, 390000,
	totalPriorExpenses(dummy), totalExpenses(dummy)
];

export type LinePoints = {
	points: { x: number; y: number }[];
	path: string;
	area: string;
};

export function buildLine(values: number[], width: number, height: number): LinePoints {
	const min = Math.min(...values);
	const max = Math.max(...values);
	const range = max - min || 1;
	const stepX = values.length > 1 ? width / (values.length - 1) : 0;
	const points = values.map((v, i) => {
		const x = i * stepX;
		const y = height - ((v - min) / range) * height;
		return { x, y };
	});
	const path = points.map((p) => `${p.x.toFixed(1)},${p.y.toFixed(1)}`).join(' ');
	const area = `0,${height} ${path} ${width.toFixed(1)},${height}`;
	return { points, path, area };
}
