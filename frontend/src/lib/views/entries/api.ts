import { apiFetch, DEV_WORKSPACE } from '$lib/api/client';
import type { JSend } from '$lib/tabs/types';
import type { Entry } from './loader';

export type CreateEntryInput = {
	date: string;
	description: string;
	debitAccountId: string;
	creditAccountId: string;
	amountCents: number;
	note?: string;
};

type PostingBody = {
	account_id: string;
	amount: number;
	note?: string;
};

export async function createEntry(input: CreateEntryInput): Promise<JSend<Entry>> {
	const note = input.note?.trim() || undefined;
	const postings: PostingBody[] = [
		{ account_id: input.debitAccountId, amount: input.amountCents, ...(note ? { note } : {}) },
		{ account_id: input.creditAccountId, amount: -input.amountCents, ...(note ? { note } : {}) }
	];

	return apiFetch<Entry>(`/workspaces/${DEV_WORKSPACE}/entries`, {
		method: 'POST',
		body: JSON.stringify({
			date: input.date,
			description: input.description,
			postings
		})
	});
}
