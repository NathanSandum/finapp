export type Line = { name: string; amount: number; children?: Line[] };

export type IncomeData = {
	period_label: string;
	prior_label: string;
	prior_short_label: string;
	revenue: Line[];
	expenses: Line[];
	prior_revenue: Line[];
	prior_expenses: Line[];
};

export type LinePoints = {
	points: { x: number; y: number }[];
	path: string;
	area: string;
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
