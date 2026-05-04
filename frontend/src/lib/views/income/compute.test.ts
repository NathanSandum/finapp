import { describe, it, expect } from 'vitest';
import {
	computeIncomeData,
	computeNetTrail,
	latestActivityMonth,
	priorPeriod,
	shiftPeriod,
	snapToGranularity
} from './compute';
import type { Account, Entry } from '../entries/loader';

const accounts: Account[] = [
	{ id: 'rev', parent_id: null, name: 'Revenue', account_type: 'revenue', currency: 'CAD' },
	{ id: 'employment', parent_id: 'rev', name: 'Employment', account_type: 'revenue', currency: 'CAD' },
	{ id: 'salary', parent_id: 'employment', name: 'Salary', account_type: 'revenue', currency: 'CAD' },
	{ id: 'exp', parent_id: null, name: 'Expenses', account_type: 'expense', currency: 'CAD' },
	{ id: 'food', parent_id: 'exp', name: 'Food', account_type: 'expense', currency: 'CAD' },
	{ id: 'rent', parent_id: 'exp', name: 'Rent', account_type: 'expense', currency: 'CAD' },
	{ id: 'assets', parent_id: null, name: 'Assets', account_type: 'asset', currency: 'CAD' },
	{ id: 'chequing', parent_id: 'assets', name: 'Chequing', account_type: 'asset', currency: 'CAD' }
];

function entry(id: string, date: string, postings: { account_id: string; amount: number }[]): Entry {
	return {
		id,
		date,
		description: '',
		postings: postings.map((p, i) => ({ id: `${id}-${i}`, account_id: p.account_id, amount: p.amount, note: '' }))
	};
}

const entries: Entry[] = [
	entry('e1', '2026-04-01', [{ account_id: 'salary', amount: -500000 }, { account_id: 'chequing', amount: 500000 }]),
	entry('e2', '2026-04-05', [{ account_id: 'food', amount: 12000 }, { account_id: 'chequing', amount: -12000 }]),
	entry('e3', '2026-04-10', [{ account_id: 'rent', amount: 180000 }, { account_id: 'chequing', amount: -180000 }]),
	entry('e4', '2026-03-01', [{ account_id: 'salary', amount: -480000 }, { account_id: 'chequing', amount: 480000 }]),
	entry('e5', '2026-03-15', [{ account_id: 'food', amount: 9000 }, { account_id: 'chequing', amount: -9000 }]),
	entry('e6', '2025-11-01', [{ account_id: 'salary', amount: -460000 }, { account_id: 'chequing', amount: 460000 }])
];

describe('computeIncomeData (month)', () => {
	it('buckets postings under direct child of root, signs flipped to display orientation', () => {
		const result = computeIncomeData(entries, accounts, 2026, 4, 'month');
		expect(result.period_label).toBe('April 2026');
		expect(result.prior_label).toBe('March 2026');
		expect(result.prior_short_label).toBe('Mar');
		expect(result.revenue).toEqual([{ name: 'Employment', amount: 500000 }]);
		expect(result.expenses).toEqual([
			{ name: 'Rent', amount: 180000 },
			{ name: 'Food', amount: 12000 }
		]);
		expect(result.prior_revenue).toEqual([{ name: 'Employment', amount: 480000 }]);
		expect(result.prior_expenses).toEqual([{ name: 'Food', amount: 9000 }]);
	});

	it('ignores asset/liability/equity postings', () => {
		const result = computeIncomeData(entries, accounts, 2026, 4, 'month');
		const allLineNames = [...result.revenue, ...result.expenses].map((l) => l.name);
		expect(allLineNames).not.toContain('Chequing');
	});

	it('excludes entries outside the period', () => {
		const result = computeIncomeData(entries, accounts, 2026, 5, 'month');
		expect(result.revenue).toEqual([]);
		expect(result.expenses).toEqual([]);
	});

	it('handles year boundary for prior month', () => {
		const result = computeIncomeData(entries, accounts, 2026, 1, 'month');
		expect(result.prior_label).toBe('December 2025');
		expect(result.prior_short_label).toBe('Dec');
	});
});

describe('computeIncomeData (quarter)', () => {
	it('aggregates a full quarter', () => {
		const result = computeIncomeData(entries, accounts, 2026, 4, 'quarter');
		expect(result.period_label).toBe('Q2 2026');
		expect(result.prior_label).toBe('Q1 2026');
		expect(result.prior_short_label).toBe('Q1');
		expect(result.revenue).toEqual([{ name: 'Employment', amount: 500000 }]);
		expect(result.expenses).toEqual([
			{ name: 'Rent', amount: 180000 },
			{ name: 'Food', amount: 12000 }
		]);
		expect(result.prior_revenue).toEqual([{ name: 'Employment', amount: 480000 }]);
		expect(result.prior_expenses).toEqual([{ name: 'Food', amount: 9000 }]);
	});

	it('snaps any month within quarter to same quarter', () => {
		const result = computeIncomeData(entries, accounts, 2026, 6, 'quarter');
		expect(result.period_label).toBe('Q2 2026');
	});

	it('handles year boundary for prior quarter', () => {
		const result = computeIncomeData(entries, accounts, 2026, 2, 'quarter');
		expect(result.prior_label).toBe('Q4 2025');
		expect(result.prior_short_label).toBe('Q4');
	});
});

describe('computeIncomeData (year)', () => {
	it('aggregates a full year', () => {
		const result = computeIncomeData(entries, accounts, 2026, 4, 'year');
		expect(result.period_label).toBe('2026');
		expect(result.prior_label).toBe('2025');
		expect(result.prior_short_label).toBe('2025');
		expect(result.revenue).toEqual([{ name: 'Employment', amount: 980000 }]);
		expect(result.expenses).toEqual([
			{ name: 'Rent', amount: 180000 },
			{ name: 'Food', amount: 21000 }
		]);
		expect(result.prior_revenue).toEqual([{ name: 'Employment', amount: 460000 }]);
	});
});

describe('computeNetTrail', () => {
	it('month: returns count buckets ending at selected period', () => {
		const result = computeNetTrail(entries, accounts, 2026, 4, 'month', 3);
		expect(result.values).toHaveLength(3);
		expect(result.labels).toEqual(['Feb', 'Mar', 'Apr']);
		expect(result.values[0]).toBe(0);
		expect(result.values[1]).toBe(480000 - 9000);
		expect(result.values[2]).toBe(500000 - 192000);
	});

	it('quarter: returns count quarterly buckets', () => {
		const result = computeNetTrail(entries, accounts, 2026, 4, 'quarter', 2);
		expect(result.labels).toEqual(['Q1', 'Q2']);
		expect(result.values[0]).toBe(480000 - 9000);
		expect(result.values[1]).toBe(500000 - 192000);
	});

	it('year: returns count yearly buckets with two-digit labels', () => {
		const result = computeNetTrail(entries, accounts, 2026, 4, 'year', 2);
		expect(result.labels).toEqual([`'25`, `'26`]);
		expect(result.values[0]).toBe(460000);
		expect(result.values[1]).toBe(980000 - 201000);
	});
});

describe('shiftPeriod', () => {
	it('month: ±1 with year wrap', () => {
		expect(shiftPeriod(2026, 1, 'month', -1)).toEqual({ year: 2025, month: 12 });
		expect(shiftPeriod(2026, 12, 'month', 1)).toEqual({ year: 2027, month: 1 });
	});

	it('quarter: ±1 quarter, snaps to quarter start month', () => {
		expect(shiftPeriod(2026, 4, 'quarter', 1)).toEqual({ year: 2026, month: 7 });
		expect(shiftPeriod(2026, 1, 'quarter', -1)).toEqual({ year: 2025, month: 10 });
	});

	it('year: ±1 year, snaps to month 1', () => {
		expect(shiftPeriod(2026, 5, 'year', 1)).toEqual({ year: 2027, month: 1 });
	});
});

describe('snapToGranularity', () => {
	it('month: no change', () => {
		expect(snapToGranularity(2026, 5, 'month')).toEqual({ year: 2026, month: 5 });
	});

	it('quarter: snaps to quarter start month', () => {
		expect(snapToGranularity(2026, 5, 'quarter')).toEqual({ year: 2026, month: 4 });
		expect(snapToGranularity(2026, 11, 'quarter')).toEqual({ year: 2026, month: 10 });
	});

	it('year: snaps to January', () => {
		expect(snapToGranularity(2026, 5, 'year')).toEqual({ year: 2026, month: 1 });
	});
});

describe('priorPeriod', () => {
	it('quarter year boundary', () => {
		expect(priorPeriod(2026, 1, 'quarter')).toEqual({ year: 2025, month: 10 });
	});
});

describe('latestActivityMonth', () => {
	it('returns the year/month of the latest entry with revenue or expense activity', () => {
		const result = latestActivityMonth(entries, accounts);
		expect(result).toEqual({ year: 2026, month: 4 });
	});

	it('falls back to current month when no qualifying entries exist', () => {
		const empty = latestActivityMonth([], accounts);
		const now = new Date();
		expect(empty).toEqual({ year: now.getUTCFullYear(), month: now.getUTCMonth() + 1 });
	});
});
