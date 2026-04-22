export function formatAmount(cents: number): string {
	const abs = Math.abs(cents);
	const dollars = Math.floor(abs / 100);
	const remainder = abs % 100;
	const sign = cents < 0 ? '-' : '';
	return `${sign}$${dollars.toLocaleString()}.${String(remainder).padStart(2, '0')}`;
}
