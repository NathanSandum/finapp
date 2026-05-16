import { describe, it, expect } from 'vitest';
import {
	computeAssetTrails,
	computeBalanceSheet,
	computeNetWorthTrail,
	latestActivityPeriod
} from './compute';
import type { Account, Entry } from '../entries/loader';

const accounts: Account[] = [
	{ id: 'assets', parent_id: null, name: 'Assets', account_type: 'asset', currency: 'CAD', role: 'assets_root' },
	{ id: 'chequing', parent_id: 'assets', name: 'Chequing', account_type: 'asset', currency: 'CAD', role: 'cash' },
	{ id: 'savings', parent_id: 'assets', name: 'Savings', account_type: 'asset', currency: 'CAD', role: null },
	{ id: 'liabilities', parent_id: null, name: 'Liabilities', account_type: 'liability', currency: 'CAD', role: 'liabilities_root' },
	{ id: 'credit_card', parent_id: 'liabilities', name: 'Credit Card', account_type: 'liability', currency: 'CAD', role: null },
	{ id: 'equity', parent_id: null, name: 'Equity', account_type: 'equity', currency: 'CAD', role: 'equity_root' },
	{ id: 'opening', parent_id: 'equity', name: 'Opening Balances', account_type: 'equity', currency: 'CAD', role: 'opening_equity' },
	{ id: 'rev', parent_id: null, name: 'Revenue', account_type: 'revenue', currency: 'CAD', role: 'revenue_root' },
	{ id: 'salary', parent_id: 'rev', name: 'Salary', account_type: 'revenue', currency: 'CAD', role: null },
	{ id: 'exp', parent_id: null, name: 'Expenses', account_type: 'expense', currency: 'CAD', role: 'expenses_root' },
	{ id: 'food', parent_id: 'exp', name: 'Food', account_type: 'expense', currency: 'CAD', role: null }
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
	entry('e1', '2026-01-01', [{ account_id: 'opening', amount: -1000000 }, { account_id: 'chequing', amount: 1000000 }]),
	entry('e2', '2026-03-01', [{ account_id: 'salary', amount: -480000 }, { account_id: 'chequing', amount: 480000 }]),
	entry('e3', '2026-03-15', [{ account_id: 'food', amount: 9000 }, { account_id: 'chequing', amount: -9000 }]),
	entry('e4', '2026-04-01', [{ account_id: 'salary', amount: -500000 }, { account_id: 'chequing', amount: 500000 }]),
	entry('e5', '2026-04-15', [{ account_id: 'food', amount: 12000 }, { account_id: 'credit_card', amount: -12000 }]),
	entry('e6', '2026-04-20', [{ account_id: 'savings', amount: 50000 }, { account_id: 'chequing', amount: -50000 }])
];

describe('computeBalanceSheet (month)', () => {
	it('returns asset and liability balances as of period end', () => {
		const result = computeBalanceSheet(entries, accounts, 2026, 4, 'month');
		expect(result.as_of_label).toBe('April 30, 2026');
		expect(result.prior_label).toBe('March 31, 2026');
		expect(result.prior_short_label).toBe('Mar');
		expect(result.assets).toEqual([
			{ name: 'Chequing', amount: 1921000 },
			{ name: 'Savings', amount: 50000 }
		]);
		expect(result.liabilities).toEqual([{ name: 'Credit Card', amount: 12000 }]);
	});

	it('flips sign on liabilities so display values are positive', () => {
		const result = computeBalanceSheet(entries, accounts, 2026, 4, 'month');
		expect(result.liabilities[0].amount).toBeGreaterThan(0);
	});

	it('returns prior balances at prior period end', () => {
		const result = computeBalanceSheet(entries, accounts, 2026, 4, 'month');
		expect(result.prior_assets).toEqual([{ name: 'Chequing', amount: 1471000 }]);
		expect(result.prior_liabilities).toEqual([]);
	});

	it('ignores revenue/expense/equity postings', () => {
		const result = computeBalanceSheet(entries, accounts, 2026, 4, 'month');
		const names = [...result.assets, ...result.liabilities].map((l) => l.name);
		expect(names).not.toContain('Salary');
		expect(names).not.toContain('Food');
		expect(names).not.toContain('Opening Balances');
	});

	it('handles year boundary for prior month', () => {
		const result = computeBalanceSheet(entries, accounts, 2026, 1, 'month');
		expect(result.prior_label).toBe('December 31, 2025');
		expect(result.prior_short_label).toBe('Dec');
	});
});

describe('computeBalanceSheet (quarter)', () => {
	it('uses end of quarter as as-of date', () => {
		const result = computeBalanceSheet(entries, accounts, 2026, 4, 'quarter');
		expect(result.as_of_label).toBe('Q2 2026');
		expect(result.prior_label).toBe('Q1 2026');
		expect(result.prior_short_label).toBe('Q1');
		expect(result.assets).toEqual([
			{ name: 'Chequing', amount: 1921000 },
			{ name: 'Savings', amount: 50000 }
		]);
	});
});

describe('computeBalanceSheet (year)', () => {
	it('uses December 31 as as-of date', () => {
		const result = computeBalanceSheet(entries, accounts, 2026, 4, 'year');
		expect(result.as_of_label).toBe('2026');
		expect(result.prior_label).toBe('2025');
		expect(result.prior_short_label).toBe('2025');
	});
});

describe('computeNetWorthTrail', () => {
	it('returns count buckets ending at selected period (month)', () => {
		const result = computeNetWorthTrail(entries, accounts, 2026, 4, 'month', 4);
		expect(result.values).toHaveLength(4);
		expect(result.labels).toEqual(['Jan', 'Feb', 'Mar', 'Apr']);
		expect(result.values[0]).toBe(1000000);
		expect(result.values[1]).toBe(1000000);
		expect(result.values[2]).toBe(1471000);
		expect(result.values[3]).toBe(1921000 + 50000 - 12000);
	});

	it('returns quarterly buckets', () => {
		const result = computeNetWorthTrail(entries, accounts, 2026, 4, 'quarter', 2);
		expect(result.labels).toEqual(['Q1', 'Q2']);
		expect(result.values[0]).toBe(1471000);
		expect(result.values[1]).toBe(1921000 + 50000 - 12000);
	});
});

describe('computeAssetTrails', () => {
	it('returns per-bucket asset trail aligned with period labels', () => {
		const result = computeAssetTrails(entries, accounts, 2026, 4, 'month', 4);
		expect(result.labels).toEqual(['Jan', 'Feb', 'Mar', 'Apr']);
		const chequing = result.trails.get('Chequing');
		expect(chequing).toEqual([1000000, 1000000, 1471000, 1921000]);
		const savings = result.trails.get('Savings');
		expect(savings).toEqual([0, 0, 0, 50000]);
	});

	it('omits liability buckets', () => {
		const result = computeAssetTrails(entries, accounts, 2026, 4, 'month', 4);
		expect(result.trails.has('Credit Card')).toBe(false);
	});
});

describe('latestActivityPeriod', () => {
	it('returns year/month of latest entry touching asset or liability', () => {
		const result = latestActivityPeriod(entries, accounts);
		expect(result).toEqual({ year: 2026, month: 4 });
	});

	it('falls back to current month when no qualifying entries exist', () => {
		const result = latestActivityPeriod([], accounts);
		const now = new Date();
		expect(result).toEqual({ year: now.getFullYear(), month: now.getMonth() + 1 });
	});
});
