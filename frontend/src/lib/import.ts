import type { Account, PostingInput } from '$lib/types';

export type { Account };

// ── Import-specific Types ───────────────────────────────────────────────────

export type MappingType =
	| 'skip'
	| 'opening_balance'
	| 'income'
	| 'expense'
	| 'deduction'
	| 'transfer'
	| 'credit_expense';

export type ColumnMapping = {
	column: string;
	type: MappingType;
	accountId: string;
	newAccountName: string;
	fundingAccountId: string;
};

export type DateRow = { date: string; cells: string[] };

export type EntryInput = {
	date: string;
	description: string;
	postings: PostingInput[];
};

// ── CSV Parsing ──────────────────────────────────────────────────────────────

const MONTHS: Record<string, string> = {
	Jan: '01',
	Feb: '02',
	Mar: '03',
	Apr: '04',
	May: '05',
	Jun: '06',
	Jul: '07',
	Aug: '08',
	Sep: '09',
	Oct: '10',
	Nov: '11',
	Dec: '12'
};

export function parseCSVLine(line: string): string[] {
	const result: string[] = [];
	let current = '';
	let inQuotes = false;
	for (const char of line) {
		if (char === '"') {
			inQuotes = !inQuotes;
		} else if (char === ',' && !inQuotes) {
			result.push(current);
			current = '';
		} else {
			current += char;
		}
	}
	result.push(current);
	return result;
}

export function parseDate(day: string, year: number): string {
	const trimmed = day.trim();
	const match = trimmed.match(/^(\w{3})\s+(\d{1,2})$/);
	if (!match) return '';
	const month = MONTHS[match[1]];
	if (!month) return '';
	return `${year}-${month}-${match[2].padStart(2, '0')}`;
}

export function parseDollars(value: string): number {
	if (!value || value.trim() === '') return 0;
	const cleaned = value.replace(/[$,]/g, '').trim();
	if (cleaned === '') return 0;
	const amount = parseFloat(cleaned);
	if (isNaN(amount)) return 0;
	return Math.round(amount * 100);
}

export { formatAmount } from '$lib/format';

// ── CSV File Parsing ─────────────────────────────────────────────────────────

export function parseCSV(
	text: string,
	year: number
): { headers: string[]; startRow: string[]; dateRows: DateRow[] } {
	const lines = text.split('\n').filter((l) => l.trim() !== '');
	if (lines.length < 2) {
		throw new Error('CSV needs header + at least one data row');
	}

	const headers = parseCSVLine(lines[0]);
	let startRow: string[] = [];
	const dateRows: DateRow[] = [];

	for (const line of lines.slice(1)) {
		const cells = parseCSVLine(line);
		const day = cells[0]?.trim();
		if (day === 'Start') {
			startRow = cells;
		} else {
			const date = parseDate(day, year);
			if (date) {
				dateRows.push({ date, cells });
			}
		}
	}

	return { headers, startRow, dateRows };
}

// ── Account Helpers ──────────────────────────────────────────────────────────

export function findAccountByName(accounts: Account[], name: string): Account | undefined {
	return accounts.find((a) => a.name === name);
}

export function findRootByType(accounts: Account[], type: string): Account | undefined {
	return accounts.find(
		(a) => a.account_type.toLowerCase() === type.toLowerCase() && !a.parent_id
	);
}

// ── Auto-Detection ───────────────────────────────────────────────────────────

export function autoDetectMappings(headers: string[], accounts: Account[]): ColumnMapping[] {
	const chequingId = findAccountByName(accounts, 'Chequing')?.id ?? '';
	const creditCardId = findAccountByName(accounts, 'Credit Card')?.id ?? '';

	return headers.slice(1).map((header) => {
		const trimmed = header.trim();

		if (trimmed.endsWith('Balance')) {
			let accountName = trimmed.replace(' Balance', '');
			if (accountName === 'Credit') accountName = 'Credit Card';
			const account = findAccountByName(accounts, accountName);
			return {
				column: trimmed,
				type: 'opening_balance' as MappingType,
				accountId: account?.id ?? '',
				newAccountName: account ? '' : accountName,
				fundingAccountId: ''
			};
		}

		if (trimmed.endsWith('on Credit')) {
			const account = findAccountByName(accounts, trimmed);
			return {
				column: trimmed,
				type: 'credit_expense' as MappingType,
				accountId: account?.id ?? '',
				newAccountName: account ? '' : trimmed,
				fundingAccountId: creditCardId
			};
		}

		if (trimmed.endsWith('Income') || trimmed === 'Benefits and Tax Credits') {
			const account = findAccountByName(accounts, trimmed);
			return {
				column: trimmed,
				type: 'income' as MappingType,
				accountId: account?.id ?? '',
				newAccountName: account ? '' : trimmed,
				fundingAccountId: chequingId
			};
		}

		const exactMatch = findAccountByName(accounts, trimmed);
		if (exactMatch && exactMatch.account_type.toLowerCase() === 'revenue') {
			return {
				column: trimmed,
				type: 'income' as MappingType,
				accountId: exactMatch.id,
				newAccountName: '',
				fundingAccountId: chequingId
			};
		}

		return {
			column: trimmed,
			type: 'expense' as MappingType,
			accountId: exactMatch?.id ?? '',
			newAccountName: exactMatch ? '' : trimmed,
			fundingAccountId: chequingId
		};
	});
}

// ── Parent Type Resolution ───────────────────────────────────────────────────

export function parentTypeForMapping(mappingType: MappingType): string {
	switch (mappingType) {
		case 'income':
			return 'revenue';
		case 'opening_balance':
			return 'asset';
		case 'transfer':
			return 'asset';
		default:
			return 'expense';
	}
}

// ── Entry Generation ─────────────────────────────────────────────────────────

export function buildOpeningBalanceEntry(
	mappings: ColumnMapping[],
	startRow: string[],
	dateRows: DateRow[],
	year: number,
	openingBalancesAccountId: string,
	accounts: Account[]
): EntryInput | null {
	const accountsById = Object.fromEntries(accounts.map((a) => [a.id, a]));
	const postings: PostingInput[] = [];

	for (let i = 0; i < mappings.length; i++) {
		const mapping = mappings[i];
		if (mapping.type !== 'opening_balance') continue;
		const cents = parseDollars(startRow[i + 1] ?? '');
		if (cents === 0) continue;

		const accountId = mapping.accountId;
		if (!accountId) continue;

		const account = accountsById[accountId];
		const isLiability = account?.account_type.toLowerCase() === 'liability';
		postings.push({ account_id: accountId, amount: isLiability ? -cents : cents });
	}

	if (postings.length === 0) return null;

	const total = postings.reduce((sum, p) => sum + p.amount, 0);
	postings.push({ account_id: openingBalancesAccountId, amount: -total });

	let openingDate = `${year - 1}-12-31`;
	if (dateRows.length > 0) {
		const d = new Date(dateRows[0].date + 'T00:00:00');
		d.setDate(d.getDate() - 1);
		openingDate = d.toISOString().slice(0, 10);
	}

	return { date: openingDate, description: 'Opening Balances', postings };
}

export function buildActivityEntries(
	mappings: ColumnMapping[],
	dateRows: DateRow[],
	resolveAccountId: (mapping: ColumnMapping) => string
): EntryInput[] {
	const entries: EntryInput[] = [];

	for (const row of dateRows) {
		for (let i = 0; i < mappings.length; i++) {
			const mapping = mappings[i];
			if (mapping.type === 'skip' || mapping.type === 'opening_balance') continue;

			const cents = parseDollars(row.cells[i + 1] ?? '');
			if (cents === 0) continue;

			const accountId = resolveAccountId(mapping);
			if (!accountId) continue;

			let postings: PostingInput[];
			if (mapping.type === 'income') {
				postings = [
					{ account_id: mapping.fundingAccountId, amount: cents },
					{ account_id: accountId, amount: -cents }
				];
			} else {
				postings = [
					{ account_id: accountId, amount: cents },
					{ account_id: mapping.fundingAccountId, amount: -cents }
				];
			}

			entries.push({ date: row.date, description: mapping.column, postings });
		}
	}

	return entries;
}

// ── New Accounts Collection ──────────────────────────────────────────────────

export function collectNewAccounts(
	mappings: ColumnMapping[],
	startRow: string[],
	dateRows: DateRow[]
): Map<string, { name: string; parentType: string }> {
	const newAccounts = new Map<string, { name: string; parentType: string }>();
	for (let i = 0; i < mappings.length; i++) {
		const mapping = mappings[i];
		if (mapping.accountId || mapping.type === 'skip' || !mapping.newAccountName) continue;

		// Only create if column has any non-zero values
		const columnIndex = i + 1; // +1 because headers[0] is Day
		let hasData = false;
		if (mapping.type === 'opening_balance') {
			hasData = parseDollars(startRow[columnIndex] ?? '') > 0;
		} else {
			hasData = dateRows.some((row) => parseDollars(row.cells[columnIndex] ?? '') > 0);
		}
		if (!hasData) continue;

		newAccounts.set(mapping.newAccountName, {
			name: mapping.newAccountName,
			parentType: parentTypeForMapping(mapping.type)
		});
	}
	return newAccounts;
}
