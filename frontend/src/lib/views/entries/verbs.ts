export type VerbId = 'spent' | 'earned' | 'contributed' | 'withdrew' | 'borrowed' | 'repaid';

export type AccountTypeFilter = 'expense' | 'revenue' | 'asset' | 'liability';

export type VerbConfig = {
	id: VerbId;
	label: string;
	counterpartyType: AccountTypeFilter;
	counterpartyPreposition: string;
	chequingSide: 'debit' | 'credit';
};

export const VERBS: VerbConfig[] = [
	{ id: 'spent', label: 'Spent', counterpartyType: 'expense', counterpartyPreposition: 'on', chequingSide: 'credit' },
	{ id: 'earned', label: 'Earned', counterpartyType: 'revenue', counterpartyPreposition: 'from', chequingSide: 'debit' },
	{ id: 'contributed', label: 'Contributed', counterpartyType: 'asset', counterpartyPreposition: 'to', chequingSide: 'credit' },
	{ id: 'withdrew', label: 'Withdrew', counterpartyType: 'asset', counterpartyPreposition: 'from', chequingSide: 'debit' },
	{ id: 'borrowed', label: 'Borrowed', counterpartyType: 'liability', counterpartyPreposition: 'via', chequingSide: 'debit' },
	{ id: 'repaid', label: 'Repaid', counterpartyType: 'liability', counterpartyPreposition: '', chequingSide: 'credit' }
];

const VERB_BY_ID: Record<VerbId, VerbConfig> = Object.fromEntries(
	VERBS.map((v) => [v.id, v])
) as Record<VerbId, VerbConfig>;

export function counterpartyTypeFor(verb: VerbId): AccountTypeFilter {
	return VERB_BY_ID[verb].counterpartyType;
}

export function verbConfig(verb: VerbId): VerbConfig {
	return VERB_BY_ID[verb];
}

export type VerbPosting = { accountId: string; amount: number };

export function buildVerbPostings(
	verb: VerbId,
	counterpartyId: string,
	chequingId: string,
	amountCents: number
): VerbPosting[] {
	const config = VERB_BY_ID[verb];
	if (config.chequingSide === 'debit') {
		return [
			{ accountId: chequingId, amount: amountCents },
			{ accountId: counterpartyId, amount: -amountCents }
		];
	}
	return [
		{ accountId: counterpartyId, amount: amountCents },
		{ accountId: chequingId, amount: -amountCents }
	];
}
