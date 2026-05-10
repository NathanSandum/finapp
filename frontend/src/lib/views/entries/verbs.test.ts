import { describe, it, expect } from 'vitest';
import { buildVerbPostings, counterpartyTypeFor, VERBS } from './verbs';

const CHEQUING = 'acct-chequing';

describe('VERBS', () => {
	it('exposes the six CF verbs', () => {
		expect(VERBS.map((v) => v.id)).toEqual([
			'spent',
			'earned',
			'contributed',
			'withdrew',
			'borrowed',
			'repaid'
		]);
	});
});

describe('counterpartyTypeFor', () => {
	it('maps verbs to account types', () => {
		expect(counterpartyTypeFor('spent')).toBe('expense');
		expect(counterpartyTypeFor('earned')).toBe('revenue');
		expect(counterpartyTypeFor('contributed')).toBe('asset');
		expect(counterpartyTypeFor('withdrew')).toBe('asset');
		expect(counterpartyTypeFor('borrowed')).toBe('liability');
		expect(counterpartyTypeFor('repaid')).toBe('liability');
	});
});

describe('buildVerbPostings', () => {
	it('Spent: DR expense, CR chequing', () => {
		const postings = buildVerbPostings('spent', 'acct-grocery', CHEQUING, 5000);
		expect(postings).toEqual([
			{ accountId: 'acct-grocery', amount: 5000 },
			{ accountId: CHEQUING, amount: -5000 }
		]);
	});

	it('Earned: DR chequing, CR revenue', () => {
		const postings = buildVerbPostings('earned', 'acct-salary', CHEQUING, 500000);
		expect(postings).toEqual([
			{ accountId: CHEQUING, amount: 500000 },
			{ accountId: 'acct-salary', amount: -500000 }
		]);
	});

	it('Contributed: DR asset, CR chequing', () => {
		const postings = buildVerbPostings('contributed', 'acct-tfsa', CHEQUING, 100000);
		expect(postings).toEqual([
			{ accountId: 'acct-tfsa', amount: 100000 },
			{ accountId: CHEQUING, amount: -100000 }
		]);
	});

	it('Withdrew: DR chequing, CR asset', () => {
		const postings = buildVerbPostings('withdrew', 'acct-tfsa', CHEQUING, 100000);
		expect(postings).toEqual([
			{ accountId: CHEQUING, amount: 100000 },
			{ accountId: 'acct-tfsa', amount: -100000 }
		]);
	});

	it('Borrowed: DR chequing, CR liability', () => {
		const postings = buildVerbPostings('borrowed', 'acct-loc', CHEQUING, 200000);
		expect(postings).toEqual([
			{ accountId: CHEQUING, amount: 200000 },
			{ accountId: 'acct-loc', amount: -200000 }
		]);
	});

	it('Repaid: DR liability, CR chequing', () => {
		const postings = buildVerbPostings('repaid', 'acct-mortgage', CHEQUING, 150000);
		expect(postings).toEqual([
			{ accountId: 'acct-mortgage', amount: 150000 },
			{ accountId: CHEQUING, amount: -150000 }
		]);
	});

	it('every verb sums to zero', () => {
		for (const verb of VERBS) {
			const postings = buildVerbPostings(verb.id, 'cp', CHEQUING, 1234);
			expect(postings.reduce((s, p) => s + p.amount, 0)).toBe(0);
		}
	});
});
