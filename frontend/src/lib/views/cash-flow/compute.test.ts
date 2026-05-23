import { describe, it, expect } from 'vitest';
import {
	computeCashFlow,
	computeNetCashTrail,
	computeSectionTrails,
	latestActivityPeriod
} from './compute';
import type { Account, Anchors, Entry } from '$lib/data';

const accounts: Account[] = [
	{ id: 'assets', parent_id: null, name: 'Assets', account_type: 'asset', currency: 'CAD', role: 'assets_root' },
	{ id: 'chequing', parent_id: 'assets', name: 'Chequing', account_type: 'asset', currency: 'CAD', role: 'cash' },
	{ id: 'savings', parent_id: 'assets', name: 'Savings', account_type: 'asset', currency: 'CAD', role: null },
	{ id: 'tfsa', parent_id: 'assets', name: 'TFSA', account_type: 'asset', currency: 'CAD', role: null },
	{ id: 'liabilities', parent_id: null, name: 'Liabilities', account_type: 'liability', currency: 'CAD', role: 'liabilities_root' },
	{ id: 'credit_card', parent_id: 'liabilities', name: 'Credit Card', account_type: 'liability', currency: 'CAD', role: null },
	{ id: 'equity', parent_id: null, name: 'Equity', account_type: 'equity', currency: 'CAD', role: 'equity_root' },
	{ id: 'opening', parent_id: 'equity', name: 'Opening Balances', account_type: 'equity', currency: 'CAD', role: 'opening_equity' },
	{ id: 'rev', parent_id: null, name: 'Revenue', account_type: 'revenue', currency: 'CAD', role: 'revenue_root' },
	{ id: 'salary', parent_id: 'rev', name: 'Salary', account_type: 'revenue', currency: 'CAD', role: null },
	{ id: 'exp', parent_id: null, name: 'Expenses', account_type: 'expense', currency: 'CAD', role: 'expenses_root' },
	{ id: 'food', parent_id: 'exp', name: 'Food', account_type: 'expense', currency: 'CAD', role: null }
];

const accountsById = new Map(accounts.map((a) => [a.id, a]));
const anchors: Anchors = {
	assets: accountsById.get('assets')!,
	liabilities: accountsById.get('liabilities')!,
	equity: accountsById.get('equity')!,
	revenue: accountsById.get('rev')!,
	expenses: accountsById.get('exp')!,
	cash: accountsById.get('chequing')!,
	opening_equity: accountsById.get('opening')!
};

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

describe('computeCashFlow (month)', () => {
	it('returns labels for selected and prior period', () => {
		const result = computeCashFlow(entries, accounts, anchors, 2026, 4, 'month');
		expect(result.period_label).toBe('April 2026');
		expect(result.prior_label).toBe('March 2026');
		expect(result.prior_short_label).toBe('Mar');
	});

	it('operating cash flow = revenue minus expenses for the period', () => {
		const result = computeCashFlow(entries, accounts, anchors, 2026, 4, 'month');
		expect(result.operating.total).toBe(488000);
	});

	it('operating section breaks out revenue and expense buckets with signed amounts', () => {
		const result = computeCashFlow(entries, accounts, anchors, 2026, 4, 'month');
		const salary = result.operating.lines.find((l) => l.name === 'Salary');
		const food = result.operating.lines.find((l) => l.name === 'Food');
		expect(salary?.amount).toBe(500000);
		expect(food?.amount).toBe(-12000);
	});

	it('investing cash flow = -delta(non-cash assets) per bucket', () => {
		const result = computeCashFlow(entries, accounts, anchors, 2026, 4, 'month');
		expect(result.investing.total).toBe(-50000);
		const savings = result.investing.lines.find((l) => l.name === 'Savings');
		expect(savings?.amount).toBe(-50000);
	});

	it('investing excludes the cash bucket itself', () => {
		const result = computeCashFlow(entries, accounts, anchors, 2026, 4, 'month');
		expect(result.investing.lines.find((l) => l.name === 'Chequing')).toBeUndefined();
	});

	it('financing cash flow = -delta(stored liability + equity) per bucket', () => {
		const result = computeCashFlow(entries, accounts, anchors, 2026, 4, 'month');
		expect(result.financing.total).toBe(12000);
		const cc = result.financing.lines.find((l) => l.name === 'Credit Card');
		expect(cc?.amount).toBe(12000);
	});

	it('equity changes flow through financing (opening balance entry)', () => {
		const result = computeCashFlow(entries, accounts, anchors, 2026, 1, 'month');
		expect(result.financing.total).toBe(1000000);
		const opening = result.financing.lines.find((l) => l.name === 'Opening Balances');
		expect(opening?.amount).toBe(1000000);
	});

	it('net change in cash = operating + investing + financing', () => {
		const result = computeCashFlow(entries, accounts, anchors, 2026, 4, 'month');
		const net = result.operating.total + result.investing.total + result.financing.total;
		expect(net).toBe(450000);
		expect(result.closing_cash - result.opening_cash).toBe(net);
	});

	it('opening_cash is cash balance at end of prior period', () => {
		const result = computeCashFlow(entries, accounts, anchors, 2026, 4, 'month');
		expect(result.opening_cash).toBe(1471000);
		expect(result.closing_cash).toBe(1921000);
	});

	it('returns prior section totals for delta comparisons', () => {
		const result = computeCashFlow(entries, accounts, anchors, 2026, 4, 'month');
		expect(result.prior_operating_total).toBe(471000);
		expect(result.prior_investing_total).toBe(0);
		expect(result.prior_financing_total).toBe(0);
	});

	it('omits zero-amount lines', () => {
		const result = computeCashFlow(entries, accounts, anchors, 2026, 3, 'month');
		expect(result.investing.lines).toEqual([]);
		expect(result.financing.lines).toEqual([]);
	});

	it('handles year boundary for prior month', () => {
		const result = computeCashFlow(entries, accounts, anchors, 2026, 1, 'month');
		expect(result.prior_label).toBe('December 2025');
		expect(result.prior_short_label).toBe('Dec');
	});
});

describe('computeCashFlow (quarter)', () => {
	it('aggregates monthly entries into quarter totals', () => {
		const result = computeCashFlow(entries, accounts, anchors, 2026, 4, 'quarter');
		expect(result.period_label).toBe('Q2 2026');
		expect(result.operating.total).toBe(488000);
	});

	it('Q1 includes opening balance equity entry in financing', () => {
		const result = computeCashFlow(entries, accounts, anchors, 2026, 1, 'quarter');
		expect(result.period_label).toBe('Q1 2026');
		expect(result.financing.total).toBe(1000000);
		expect(result.operating.total).toBe(471000);
	});
});

describe('computeNetCashTrail', () => {
	it('returns net cash change per period (month)', () => {
		const result = computeNetCashTrail(entries, accounts, anchors, 2026, 4, 'month', 4);
		expect(result.labels).toEqual(['Jan', 'Feb', 'Mar', 'Apr']);
		expect(result.values).toEqual([1000000, 0, 471000, 450000]);
	});

	it('returns quarterly net cash change buckets', () => {
		const result = computeNetCashTrail(entries, accounts, anchors, 2026, 4, 'quarter', 2);
		expect(result.labels).toEqual(['Q1', 'Q2']);
		expect(result.values).toEqual([1471000, 450000]);
	});
});

describe('computeSectionTrails', () => {
	it('returns operating/investing/financing totals per period', () => {
		const result = computeSectionTrails(entries, accounts, anchors, 2026, 4, 'month', 4);
		expect(result.labels).toEqual(['Jan', 'Feb', 'Mar', 'Apr']);
		expect(result.operating).toEqual([0, 0, 471000, 488000]);
		expect(result.investing).toEqual([0, 0, 0, -50000]);
		expect(result.financing).toEqual([1000000, 0, 0, 12000]);
	});
});

describe('latestActivityPeriod', () => {
	it('returns year/month of latest entry', () => {
		const result = latestActivityPeriod(entries);
		expect(result).toEqual({ year: 2026, month: 4 });
	});

	it('falls back to current month when no entries exist', () => {
		const result = latestActivityPeriod([]);
		const now = new Date();
		expect(result).toEqual({ year: now.getFullYear(), month: now.getMonth() + 1 });
	});
});
