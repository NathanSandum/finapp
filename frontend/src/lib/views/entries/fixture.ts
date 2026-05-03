export type Posting = {
	id: string;
	account_id: string;
	amount: number;
	note: string;
};

export type Entry = {
	id: string;
	date: string;
	description: string;
	postings: Posting[];
};

export const accountLabels: Record<string, string> = {
	'acc-checking': 'Assets:Checking',
	'acc-savings': 'Assets:Savings',
	'acc-credit': 'Liabilities:Credit Card',
	'acc-salary': 'Income:Salary',
	'acc-rent': 'Expenses:Rent',
	'acc-groceries': 'Expenses:Groceries',
	'acc-utilities': 'Expenses:Utilities',
	'acc-dining': 'Expenses:Dining',
	'acc-transport': 'Expenses:Transport',
	'acc-entertainment': 'Expenses:Entertainment',
	'acc-investments': 'Assets:Investments',
	'acc-tax': 'Expenses:Tax'
};

export const fixtureEntries: Entry[] = [
	{
		id: 'e-001',
		date: '2026-05-01',
		description: 'Salary — May',
		postings: [
			{ id: 'p-001a', account_id: 'acc-checking', amount: 540000, note: '' },
			{ id: 'p-001b', account_id: 'acc-tax', amount: 120000, note: 'PAYE' },
			{ id: 'p-001c', account_id: 'acc-salary', amount: -660000, note: '' }
		]
	},
	{
		id: 'e-002',
		date: '2026-05-01',
		description: 'Rent — May',
		postings: [
			{ id: 'p-002a', account_id: 'acc-rent', amount: 180000, note: '' },
			{ id: 'p-002b', account_id: 'acc-checking', amount: -180000, note: '' }
		]
	},
	{
		id: 'e-003',
		date: '2026-05-02',
		description: 'Whole Foods',
		postings: [
			{ id: 'p-003a', account_id: 'acc-groceries', amount: 8742, note: 'weekly shop' },
			{ id: 'p-003b', account_id: 'acc-credit', amount: -8742, note: '' }
		]
	},
	{
		id: 'e-004',
		date: '2026-05-02',
		description: 'Coffee',
		postings: [
			{ id: 'p-004a', account_id: 'acc-dining', amount: 425, note: '' },
			{ id: 'p-004b', account_id: 'acc-credit', amount: -425, note: '' }
		]
	},
	{
		id: 'e-005',
		date: '2026-05-03',
		description: 'Uber to airport',
		postings: [
			{ id: 'p-005a', account_id: 'acc-transport', amount: 4200, note: '' },
			{ id: 'p-005b', account_id: 'acc-credit', amount: -4200, note: '' }
		]
	},
	{
		id: 'e-006',
		date: '2026-05-04',
		description: 'Power bill',
		postings: [
			{ id: 'p-006a', account_id: 'acc-utilities', amount: 11250, note: 'Genesis' },
			{ id: 'p-006b', account_id: 'acc-checking', amount: -11250, note: '' }
		]
	},
	{
		id: 'e-007',
		date: '2026-05-05',
		description: 'Transfer to savings',
		postings: [
			{ id: 'p-007a', account_id: 'acc-savings', amount: 100000, note: '' },
			{ id: 'p-007b', account_id: 'acc-checking', amount: -100000, note: '' }
		]
	},
	{
		id: 'e-008',
		date: '2026-05-06',
		description: 'Cinema',
		postings: [
			{ id: 'p-008a', account_id: 'acc-entertainment', amount: 2400, note: '' },
			{ id: 'p-008b', account_id: 'acc-credit', amount: -2400, note: '' }
		]
	},
	{
		id: 'e-009',
		date: '2026-05-07',
		description: 'Petrol',
		postings: [
			{ id: 'p-009a', account_id: 'acc-transport', amount: 9800, note: 'Z station' },
			{ id: 'p-009b', account_id: 'acc-credit', amount: -9800, note: '' }
		]
	},
	{
		id: 'e-010',
		date: '2026-05-08',
		description: 'Countdown',
		postings: [
			{ id: 'p-010a', account_id: 'acc-groceries', amount: 12350, note: '' },
			{ id: 'p-010b', account_id: 'acc-credit', amount: -12350, note: '' }
		]
	},
	{
		id: 'e-011',
		date: '2026-05-09',
		description: 'Brokerage deposit',
		postings: [
			{ id: 'p-011a', account_id: 'acc-investments', amount: 250000, note: 'monthly DCA' },
			{ id: 'p-011b', account_id: 'acc-checking', amount: -250000, note: '' }
		]
	},
	{
		id: 'e-012',
		date: '2026-05-10',
		description: 'Dinner — Saan',
		postings: [
			{ id: 'p-012a', account_id: 'acc-dining', amount: 14600, note: 'birthday' },
			{ id: 'p-012b', account_id: 'acc-credit', amount: -14600, note: '' }
		]
	},
	{
		id: 'e-013',
		date: '2026-05-11',
		description: 'Internet',
		postings: [
			{ id: 'p-013a', account_id: 'acc-utilities', amount: 8900, note: '2degrees' },
			{ id: 'p-013b', account_id: 'acc-checking', amount: -8900, note: '' }
		]
	},
	{
		id: 'e-014',
		date: '2026-05-12',
		description: 'Spotify',
		postings: [
			{ id: 'p-014a', account_id: 'acc-entertainment', amount: 1499, note: '' },
			{ id: 'p-014b', account_id: 'acc-credit', amount: -1499, note: '' }
		]
	},
	{
		id: 'e-015',
		date: '2026-05-13',
		description: 'Pak\u2019nSave',
		postings: [
			{ id: 'p-015a', account_id: 'acc-groceries', amount: 9870, note: '' },
			{ id: 'p-015b', account_id: 'acc-credit', amount: -9870, note: '' }
		]
	},
	{
		id: 'e-016',
		date: '2026-05-14',
		description: 'Bus card top-up',
		postings: [
			{ id: 'p-016a', account_id: 'acc-transport', amount: 4000, note: 'AT HOP' },
			{ id: 'p-016b', account_id: 'acc-checking', amount: -4000, note: '' }
		]
	},
	{
		id: 'e-017',
		date: '2026-05-15',
		description: 'Salary advance — overtime',
		postings: [
			{ id: 'p-017a', account_id: 'acc-checking', amount: 80000, note: '' },
			{ id: 'p-017b', account_id: 'acc-tax', amount: 20000, note: 'PAYE' },
			{ id: 'p-017c', account_id: 'acc-salary', amount: -100000, note: '' }
		]
	},
	{
		id: 'e-018',
		date: '2026-05-16',
		description: 'Brunch — Best Ugly',
		postings: [
			{ id: 'p-018a', account_id: 'acc-dining', amount: 3850, note: '' },
			{ id: 'p-018b', account_id: 'acc-credit', amount: -3850, note: '' }
		]
	},
	{
		id: 'e-019',
		date: '2026-05-17',
		description: 'Credit card payment',
		postings: [
			{ id: 'p-019a', account_id: 'acc-credit', amount: 75000, note: '' },
			{ id: 'p-019b', account_id: 'acc-checking', amount: -75000, note: '' }
		]
	},
	{
		id: 'e-020',
		date: '2026-05-18',
		description: 'Concert ticket',
		postings: [
			{ id: 'p-020a', account_id: 'acc-entertainment', amount: 18900, note: 'Spark Arena' },
			{ id: 'p-020b', account_id: 'acc-credit', amount: -18900, note: '' }
		]
	},
	{
		id: 'e-021',
		date: '2026-05-19',
		description: 'Farmers market',
		postings: [
			{ id: 'p-021a', account_id: 'acc-groceries', amount: 4720, note: 'cash' },
			{ id: 'p-021b', account_id: 'acc-checking', amount: -4720, note: '' }
		]
	},
	{
		id: 'e-022',
		date: '2026-05-20',
		description: 'Gas bill',
		postings: [
			{ id: 'p-022a', account_id: 'acc-utilities', amount: 6300, note: '' },
			{ id: 'p-022b', account_id: 'acc-checking', amount: -6300, note: '' }
		]
	},
	{
		id: 'e-023',
		date: '2026-05-21',
		description: 'Lunch — work',
		postings: [
			{ id: 'p-023a', account_id: 'acc-dining', amount: 1850, note: '' },
			{ id: 'p-023b', account_id: 'acc-credit', amount: -1850, note: '' }
		]
	},
	{
		id: 'e-024',
		date: '2026-05-22',
		description: 'Train fares',
		postings: [
			{ id: 'p-024a', account_id: 'acc-transport', amount: 1240, note: '' },
			{ id: 'p-024b', account_id: 'acc-checking', amount: -1240, note: '' }
		]
	},
	{
		id: 'e-025',
		date: '2026-05-23',
		description: 'New Lynn Countdown',
		postings: [
			{ id: 'p-025a', account_id: 'acc-groceries', amount: 7430, note: '' },
			{ id: 'p-025b', account_id: 'acc-credit', amount: -7430, note: '' }
		]
	},
	{
		id: 'e-026',
		date: '2026-05-24',
		description: 'Netflix',
		postings: [
			{ id: 'p-026a', account_id: 'acc-entertainment', amount: 2399, note: '' },
			{ id: 'p-026b', account_id: 'acc-credit', amount: -2399, note: '' }
		]
	},
	{
		id: 'e-027',
		date: '2026-05-25',
		description: 'Brokerage deposit',
		postings: [
			{ id: 'p-027a', account_id: 'acc-investments', amount: 50000, note: 'top-up' },
			{ id: 'p-027b', account_id: 'acc-savings', amount: -50000, note: '' }
		]
	},
	{
		id: 'e-028',
		date: '2026-05-26',
		description: 'Dentist',
		postings: [
			{ id: 'p-028a', account_id: 'acc-utilities', amount: 28000, note: 'health misc' },
			{ id: 'p-028b', account_id: 'acc-checking', amount: -28000, note: '' }
		]
	},
	{
		id: 'e-029',
		date: '2026-05-27',
		description: 'Dinner — Cazador',
		postings: [
			{ id: 'p-029a', account_id: 'acc-dining', amount: 16800, note: '' },
			{ id: 'p-029b', account_id: 'acc-credit', amount: -16800, note: '' }
		]
	},
	{
		id: 'e-030',
		date: '2026-05-28',
		description: 'Annual insurance',
		postings: [
			{ id: 'p-030a', account_id: 'acc-utilities', amount: 96000, note: 'AA contents' },
			{ id: 'p-030b', account_id: 'acc-checking', amount: -96000, note: '' }
		]
	}
];

export function formatAmount(cents: number): string {
	const sign = cents < 0 ? '-' : '';
	const abs = Math.abs(cents);
	const dollars = Math.floor(abs / 100);
	const c = (abs % 100).toString().padStart(2, '0');
	return `${sign}$${dollars.toLocaleString()}.${c}`;
}

export function accountLabel(id: string): string {
	return accountLabels[id] ?? id;
}
