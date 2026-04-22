export function currentMonthPeriod(): { periodStart: string; periodEnd: string } {
	const now = new Date();
	const periodStart = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}-01`;
	const lastDay = new Date(now.getFullYear(), now.getMonth() + 1, 0);
	const periodEnd = `${lastDay.getFullYear()}-${String(lastDay.getMonth() + 1).padStart(2, '0')}-${String(lastDay.getDate()).padStart(2, '0')}`;
	return { periodStart, periodEnd };
}
