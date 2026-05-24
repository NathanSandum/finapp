import { apiFetch } from './client';
import { workspaceState } from '$lib/workspace/state.svelte';
import type { JSend } from '$lib/tabs/types';
import type { Entry, Posting } from '$lib/data/types';

type WirePosting = { id: string; account_id: string; amount: number; note: string };
type WireEntry = { id: string; date: string; description: string; postings: WirePosting[] };

function toPosting(wire: WirePosting): Posting {
	return {
		id: wire.id,
		account_id: wire.account_id,
		amount: wire.amount,
		note: wire.note
	};
}

function toEntry(wire: WireEntry): Entry {
	return {
		id: wire.id,
		date: wire.date,
		description: wire.description,
		postings: wire.postings.map(toPosting)
	};
}

export async function fetchAll(): Promise<JSend<Entry[]>> {
	const result = await apiFetch<{ entries: WireEntry[] }>(
		`/workspaces/${workspaceState.id()}/entries`
	);
	if (result.status !== 'success') return result;
	return { status: 'success', data: result.data.entries.map(toEntry) };
}

export type CreatePostingInput = { accountId: string; amount: number };
export type CreateInput = {
	date: string;
	description: string;
	postings: CreatePostingInput[];
	note?: string;
};

export async function create(input: CreateInput): Promise<JSend<Entry>> {
	const note = input.note?.trim() || undefined;
	const postings = input.postings.map((p) => ({
		account_id: p.accountId,
		amount: p.amount,
		...(note ? { note } : {})
	}));
	const result = await apiFetch<WireEntry>(
		`/workspaces/${workspaceState.id()}/entries`,
		{
			method: 'POST',
			body: JSON.stringify({
				date: input.date,
				description: input.description,
				postings
			})
		}
	);
	if (result.status !== 'success') return result;
	return { status: 'success', data: toEntry(result.data) };
}

export function remove(id: string): Promise<JSend<null>> {
	return apiFetch<null>(`/workspaces/${workspaceState.id()}/entries/${id}`, {
		method: 'DELETE'
	});
}
