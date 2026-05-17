import { discreteState } from './ui/discrete.svelte';

const CENSORED = '•••';

export function fmt(cents: number): string {
	const sign = cents < 0 ? '-' : '';
	if (discreteState.on) return `${sign}$${CENSORED}`;
	const abs = Math.abs(cents);
	const dollars = Math.floor(abs / 100);
	const c = (abs % 100).toString().padStart(2, '0');
	return `${sign}$${dollars.toLocaleString()}.${c}`;
}

export function delta(current: number, prior: number): string {
	const diff = current - prior;
	const sign = diff > 0 ? '+' : diff < 0 ? '-' : '';
	if (discreteState.on) return `${sign}$${CENSORED}`;
	const abs = Math.abs(diff);
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

export function shortFmt(cents: number): string {
	const abs = Math.abs(cents);
	const sign = cents < 0 ? '-' : '';
	if (discreteState.on) return `${sign}$${CENSORED}`;
	if (abs >= 1_000_000_00) return `${sign}$${(abs / 1_000_000_00).toFixed(1)}M`;
	if (abs >= 1_000_00) return `${sign}$${Math.round(abs / 1_000_00)}k`;
	return `${sign}$${Math.round(abs / 100)}`;
}
