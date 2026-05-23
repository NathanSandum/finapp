import { apiFetch } from '$lib/api/client';
import { workspaceState } from '$lib/workspace/state.svelte';
import type { JSend } from '$lib/tabs/types';
import type { Entry } from '$lib/data';

export type EntryPostingInput = {
	accountId: string;
	amount: number;
};

export type CreateEntryInput = {
	date: string;
	description: string;
	postings: EntryPostingInput[];
	note?: string;
};

type PostingBody = {
	account_id: string;
	amount: number;
	note?: string;
};

export async function createEntry(input: CreateEntryInput): Promise<JSend<Entry>> {
	const note = input.note?.trim() || undefined;
	const postings: PostingBody[] = input.postings.map((p) => ({
		account_id: p.accountId,
		amount: p.amount,
		...(note ? { note } : {})
	}));

	return apiFetch<Entry>(`/workspaces/${workspaceState.id()}/entries`, {
		method: 'POST',
		body: JSON.stringify({
			date: input.date,
			description: input.description,
			postings
		})
	});
}
