import { describe, it, expect } from 'vitest';
import {
	parseCSVLine,
	parseDate,
	parseDollars,
	parseCSV,
	autoDetectMappings,
	buildOpeningBalanceEntry,
	buildActivityEntries,
	collectNewAccounts,
	findRootByType,
	parentTypeForMapping,
	type Account,
	type ColumnMapping
} from './import';

// ── Test Data ────────────────────────────────────────────────────────────────

const CSV_TEXT = `Day,Chequing Balance,Savings Balance,Savings Contributions,Employment Income,Benefits and Tax Credits,Income Tax,CPP,EI,Investment Income,Financing Income,Rent,Household Expenses,Eating Out,Cafes,Books,Workspace Supplies,Other,Credit Payment,Transit,Haircut,Banking Fees,Credit Balance,Recreation on Credit,Entrepreneurship on Credit,Workplace on Credit,Subscriptions on Credit
Start,"$3,968.77","$8,105.49",,,,,,,,,,,,,,,,,,,,55.45,,,,
Jan 01,"$3,968.77","$8,105.49",$0.00,$0.00,$0.00,$0.00,$0.00,$0.00,$0.00,$0.00,$0.00,$0.00,$0.00,$0.00,$0.00,$0.00,$0.00,$0.00,$0.00,$0.00,$0.00,$64.32,$0.00,$8.87,$0.00,$0.00
Jan 02,"$3,968.77","$8,105.49",$0.00,$0.00,$0.00,$0.00,$0.00,$0.00,$0.00,$0.00,$0.00,$0.00,$5.50,$0.00,$0.00,$0.00,$0.00,$0.00,$0.00,$0.00,$0.00,$64.32,$0.00,$0.00,$0.00,$0.00`;

const DEFAULT_ACCOUNTS: Account[] = [
	{ id: 'a1', name: 'Assets', account_type: 'Asset', parent_id: null },
	{ id: 'a2', name: 'Liabilities', account_type: 'Liability', parent_id: null },
	{ id: 'a3', name: 'Equity', account_type: 'Equity', parent_id: null },
	{ id: 'a4', name: 'Revenue', account_type: 'Revenue', parent_id: null },
	{ id: 'a5', name: 'Expenses', account_type: 'Expense', parent_id: null },
	{ id: 'a6', name: 'Chequing', account_type: 'Asset', parent_id: 'a1' },
	{ id: 'a7', name: 'Savings', account_type: 'Asset', parent_id: 'a1' },
	{ id: 'a8', name: 'TFSA', account_type: 'Asset', parent_id: 'a1' },
	{ id: 'a9', name: 'RRSP', account_type: 'Asset', parent_id: 'a1' },
	{ id: 'a10', name: 'Credit Card', account_type: 'Liability', parent_id: 'a2' },
	{ id: 'a11', name: 'Employment', account_type: 'Revenue', parent_id: 'a4' },
	{ id: 'a12', name: 'Food', account_type: 'Expense', parent_id: 'a5' },
	{ id: 'a13', name: 'Rent', account_type: 'Expense', parent_id: 'a5' },
	{ id: 'a14', name: 'Groceries', account_type: 'Expense', parent_id: 'a5' },
	{ id: 'a15', name: 'Dining', account_type: 'Expense', parent_id: 'a5' },
	{ id: 'a16', name: 'Transit', account_type: 'Expense', parent_id: 'a5' },
	{ id: 'a17', name: 'Utilities', account_type: 'Expense', parent_id: 'a5' },
	{ id: 'a18', name: 'Entertainment', account_type: 'Expense', parent_id: 'a5' },
	{ id: 'a19', name: 'Opening Balances', account_type: 'Equity', parent_id: 'a3' }
];

// ── CSV Parsing ──────────────────────────────────────────────────────────────

describe('parseCSVLine', () => {
	it('splits simple comma-separated values', () => {
		expect(parseCSVLine('a,b,c')).toEqual(['a', 'b', 'c']);
	});

	it('handles quoted fields with commas', () => {
		expect(parseCSVLine('Start,"$3,968.77","$8,105.49"')).toEqual([
			'Start',
			'$3,968.77',
			'$8,105.49'
		]);
	});

	it('handles empty fields', () => {
		expect(parseCSVLine('a,,c,')).toEqual(['a', '', 'c', '']);
	});
});

describe('parseDate', () => {
	it('parses "Jan 01" format', () => {
		expect(parseDate('Jan 01', 2026)).toBe('2026-01-01');
	});

	it('parses "Dec 31" format', () => {
		expect(parseDate('Dec 31', 2025)).toBe('2025-12-31');
	});

	it('pads single-digit days', () => {
		expect(parseDate('Mar 5', 2026)).toBe('2026-03-05');
	});

	it('returns empty for "Start"', () => {
		expect(parseDate('Start', 2026)).toBe('');
	});

	it('returns empty for invalid input', () => {
		expect(parseDate('garbage', 2026)).toBe('');
	});
});

describe('parseDollars', () => {
	it('parses simple dollar amount', () => {
		expect(parseDollars('$5.50')).toBe(550);
	});

	it('parses amount with commas', () => {
		expect(parseDollars('$3,968.77')).toBe(396877);
	});

	it('parses amount without dollar sign', () => {
		expect(parseDollars('55.45')).toBe(5545);
	});

	it('returns 0 for empty string', () => {
		expect(parseDollars('')).toBe(0);
	});

	it('returns 0 for $0.00', () => {
		expect(parseDollars('$0.00')).toBe(0);
	});
});

describe('parseCSV', () => {
	it('separates headers, start row, and date rows', () => {
		const result = parseCSV(CSV_TEXT, 2026);

		expect(result.headers[0]).toBe('Day');
		expect(result.headers[1]).toBe('Chequing Balance');
		// 26 named columns + 1 trailing empty from trailing comma
		expect(result.headers.length).toBe(27);

		expect(result.startRow[0]).toBe('Start');
		expect(result.startRow[1]).toBe('$3,968.77');

		expect(result.dateRows.length).toBe(2);
		expect(result.dateRows[0].date).toBe('2026-01-01');
		expect(result.dateRows[1].date).toBe('2026-01-02');
	});

	it('throws for too-short CSV', () => {
		expect(() => parseCSV('just a header', 2026)).toThrow('CSV needs header');
	});
});

// ── Auto-Detection ───────────────────────────────────────────────────────────

describe('autoDetectMappings', () => {
	const { headers } = parseCSV(CSV_TEXT, 2026);
	const mappings = autoDetectMappings(headers, DEFAULT_ACCOUNTS);

	it('detects balance columns as opening_balance', () => {
		const chequingBalance = mappings.find((m) => m.column === 'Chequing Balance');
		expect(chequingBalance?.type).toBe('opening_balance');
		expect(chequingBalance?.accountId).toBe('a6'); // Chequing account
		expect(chequingBalance?.newAccountName).toBe('');
	});

	it('maps Credit Balance to Credit Card account', () => {
		const creditBalance = mappings.find((m) => m.column === 'Credit Balance');
		expect(creditBalance?.type).toBe('opening_balance');
		expect(creditBalance?.accountId).toBe('a10'); // Credit Card
	});

	it('detects "on Credit" columns as credit_expense', () => {
		const recreation = mappings.find((m) => m.column === 'Recreation on Credit');
		expect(recreation?.type).toBe('credit_expense');
		expect(recreation?.fundingAccountId).toBe('a10'); // Credit Card
	});

	it('detects income columns', () => {
		const employment = mappings.find((m) => m.column === 'Employment Income');
		expect(employment?.type).toBe('income');
		expect(employment?.fundingAccountId).toBe('a6'); // Chequing

		const benefits = mappings.find((m) => m.column === 'Benefits and Tax Credits');
		expect(benefits?.type).toBe('income');
	});

	it('does not detect "Income Tax" as income', () => {
		const incomeTax = mappings.find((m) => m.column === 'Income Tax');
		expect(incomeTax?.type).toBe('expense');
	});

	it('matches existing expense accounts by name', () => {
		const rent = mappings.find((m) => m.column === 'Rent');
		expect(rent?.type).toBe('expense');
		expect(rent?.accountId).toBe('a13'); // Rent account
		expect(rent?.newAccountName).toBe('');
	});

	it('marks unmatched columns as expense with new account', () => {
		const cafes = mappings.find((m) => m.column === 'Cafes');
		expect(cafes?.type).toBe('expense');
		expect(cafes?.accountId).toBe('');
		expect(cafes?.newAccountName).toBe('Cafes');
	});

	it('sets Chequing as default funding for expense columns', () => {
		const cafes = mappings.find((m) => m.column === 'Cafes');
		expect(cafes?.fundingAccountId).toBe('a6'); // Chequing
	});
});

// ── findRootByType ───────────────────────────────────────────────────────────

describe('findRootByType', () => {
	it('finds root with matching type (case-insensitive)', () => {
		expect(findRootByType(DEFAULT_ACCOUNTS, 'asset')?.id).toBe('a1');
		expect(findRootByType(DEFAULT_ACCOUNTS, 'Asset')?.id).toBe('a1');
		expect(findRootByType(DEFAULT_ACCOUNTS, 'revenue')?.id).toBe('a4');
		expect(findRootByType(DEFAULT_ACCOUNTS, 'expense')?.id).toBe('a5');
	});

	it('does not return child accounts', () => {
		// Chequing is Asset type but has a parent
		const result = findRootByType(DEFAULT_ACCOUNTS, 'asset');
		expect(result?.name).toBe('Assets');
		expect(result?.parent_id).toBeNull();
	});

	it('works when parent_id is null (not undefined)', () => {
		// API returns null for root accounts — verify our check handles it
		const accounts: Account[] = [
			{ id: 'r1', name: 'Assets', account_type: 'asset', parent_id: null }
		];
		expect(findRootByType(accounts, 'asset')?.id).toBe('r1');
	});

	it('returns undefined for empty accounts list', () => {
		expect(findRootByType([], 'asset')).toBeUndefined();
	});
});

// ── collectNewAccounts ───────────────────────────────────────────────────────

describe('collectNewAccounts', () => {
	it('collects accounts that need creation (only if column has data)', () => {
		const { headers, startRow, dateRows } = parseCSV(CSV_TEXT, 2026);
		const mappings = autoDetectMappings(headers, DEFAULT_ACCOUNTS);
		const newAccounts = collectNewAccounts(mappings, startRow, dateRows);

		// Matched to existing accounts → not new
		expect(newAccounts.has('Chequing')).toBe(false);
		expect(newAccounts.has('Savings')).toBe(false);
		expect(newAccounts.has('Credit Card')).toBe(false);
		expect(newAccounts.has('Rent')).toBe(false);
		expect(newAccounts.has('Transit')).toBe(false);

		// Has data → created
		expect(newAccounts.has('Entrepreneurship on Credit')).toBe(true);
		expect(newAccounts.has('Eating Out')).toBe(true);

		// All zeros in test data → NOT created
		expect(newAccounts.has('Employment Income')).toBe(false);
		expect(newAccounts.has('Cafes')).toBe(false);
	});

	it('assigns correct parent types', () => {
		const { headers, startRow, dateRows } = parseCSV(CSV_TEXT, 2026);
		const mappings = autoDetectMappings(headers, DEFAULT_ACCOUNTS);
		const newAccounts = collectNewAccounts(mappings, startRow, dateRows);

		expect(newAccounts.get('Eating Out')?.parentType).toBe('expense');
		expect(newAccounts.get('Entrepreneurship on Credit')?.parentType).toBe('expense');
	});
});

// ── parentTypeForMapping ─────────────────────────────────────────────────────

describe('parentTypeForMapping', () => {
	it('maps income to revenue', () => {
		expect(parentTypeForMapping('income')).toBe('revenue');
	});

	it('maps expense types to expense', () => {
		expect(parentTypeForMapping('expense')).toBe('expense');
		expect(parentTypeForMapping('deduction')).toBe('expense');
		expect(parentTypeForMapping('credit_expense')).toBe('expense');
	});

	it('maps opening_balance and transfer to asset', () => {
		expect(parentTypeForMapping('opening_balance')).toBe('asset');
		expect(parentTypeForMapping('transfer')).toBe('asset');
	});
});

// ── Opening Balance Entry ────────────────────────────────────────────────────

describe('buildOpeningBalanceEntry', () => {
	it('creates entry with correct postings for assets and liability', () => {
		const { headers, startRow, dateRows } = parseCSV(CSV_TEXT, 2026);
		const mappings = autoDetectMappings(headers, DEFAULT_ACCOUNTS);
		const entry = buildOpeningBalanceEntry(
			mappings,
			startRow,
			dateRows,
			2026,
			'a19',
			DEFAULT_ACCOUNTS
		);

		expect(entry).not.toBeNull();
		expect(entry!.description).toBe('Opening Balances');
		// Dated day before first activity
		expect(entry!.date).toBe('2025-12-31');

		// Chequing: +396877 (debit asset)
		const chequing = entry!.postings.find((p) => p.account_id === 'a6');
		expect(chequing?.amount).toBe(396877);

		// Savings: +810549 (debit asset)
		const savings = entry!.postings.find((p) => p.account_id === 'a7');
		expect(savings?.amount).toBe(810549);

		// Credit Card: -5545 (credit liability)
		const credit = entry!.postings.find((p) => p.account_id === 'a10');
		expect(credit?.amount).toBe(-5545);

		// Opening Balances equity: offsetting amount
		const openingBal = entry!.postings.find((p) => p.account_id === 'a19');
		expect(openingBal).toBeDefined();

		// All postings sum to zero
		const sum = entry!.postings.reduce((s, p) => s + p.amount, 0);
		expect(sum).toBe(0);
	});

	it('returns null when no balance data in start row', () => {
		const mappings: ColumnMapping[] = [
			{
				column: 'Chequing Balance',
				type: 'opening_balance',
				accountId: 'a6',
				newAccountName: '',
				fundingAccountId: ''
			}
		];
		const entry = buildOpeningBalanceEntry(mappings, ['Start', ''], [], 2026, 'a19', DEFAULT_ACCOUNTS);
		expect(entry).toBeNull();
	});
});

// ── Activity Entries ─────────────────────────────────────────────────────────

describe('buildActivityEntries', () => {
	it('creates entries for non-zero activity cells', () => {
		const { headers, dateRows } = parseCSV(CSV_TEXT, 2026);
		const mappings = autoDetectMappings(headers, DEFAULT_ACCOUNTS);
		const entries = buildActivityEntries(mappings, dateRows, (m) => m.accountId || 'new-id');

		// Jan 01 has $8.87 Entrepreneurship on Credit, $64.32 Credit Balance (but that's opening_balance → skipped)
		// Jan 02 has $5.50 Eating Out, $64.32 Credit Balance (skipped)
		expect(entries.length).toBeGreaterThan(0);

		// Each entry has exactly 2 postings summing to zero
		for (const entry of entries) {
			expect(entry.postings.length).toBe(2);
			const sum = entry.postings.reduce((s, p) => s + p.amount, 0);
			expect(sum).toBe(0);
		}
	});

	it('skips zero-amount cells', () => {
		const { headers, dateRows } = parseCSV(CSV_TEXT, 2026);
		const mappings = autoDetectMappings(headers, DEFAULT_ACCOUNTS);
		const entries = buildActivityEntries(mappings, dateRows, (m) => m.accountId || 'new-id');

		// Should not have entries for $0.00 cells
		const zeroEntries = entries.filter((e) =>
			e.postings.some((p) => p.amount === 0)
		);
		expect(zeroEntries.length).toBe(0);
	});

	it('uses correct signs for income (credit revenue, debit funding)', () => {
		const mappings: ColumnMapping[] = [
			{
				column: 'Employment Income',
				type: 'income',
				accountId: 'rev-1',
				newAccountName: '',
				fundingAccountId: 'a6'
			}
		];
		const dateRows = [{ date: '2026-01-15', cells: ['Jan 15', '$1,000.00'] }];
		const entries = buildActivityEntries(mappings, dateRows, (m) => m.accountId);

		expect(entries.length).toBe(1);
		// Funding (Chequing) gets debited (positive)
		expect(entries[0].postings[0]).toEqual({ account_id: 'a6', amount: 100000 });
		// Revenue gets credited (negative)
		expect(entries[0].postings[1]).toEqual({ account_id: 'rev-1', amount: -100000 });
	});

	it('uses correct signs for expense (debit expense, credit funding)', () => {
		const mappings: ColumnMapping[] = [
			{
				column: 'Rent',
				type: 'expense',
				accountId: 'a13',
				newAccountName: '',
				fundingAccountId: 'a6'
			}
		];
		const dateRows = [{ date: '2026-01-01', cells: ['Jan 01', '$500.00'] }];
		const entries = buildActivityEntries(mappings, dateRows, (m) => m.accountId);

		expect(entries.length).toBe(1);
		// Expense gets debited (positive)
		expect(entries[0].postings[0]).toEqual({ account_id: 'a13', amount: 50000 });
		// Funding (Chequing) gets credited (negative)
		expect(entries[0].postings[1]).toEqual({ account_id: 'a6', amount: -50000 });
	});
});
