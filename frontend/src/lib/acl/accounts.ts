import { apiFetch } from './client';
import { workspaceState } from '$lib/workspace/state.svelte';
import type { JSend } from '$lib/tabs/types';
import type { Account, AccountRole } from '$lib/data/types';

type WireAccount = {
	id: string;
	parent_id: string | null;
	name: string;
	account_type: string;
	currency: string;
	role: AccountRole | null;
};

function toAccount(wire: WireAccount): Account {
	return {
		id: wire.id,
		parent_id: wire.parent_id,
		name: wire.name,
		account_type: wire.account_type,
		currency: wire.currency,
		role: wire.role
	};
}

export async function fetchAll(): Promise<JSend<Account[]>> {
	const result = await apiFetch<{ accounts: WireAccount[] }>(
		`/workspaces/${workspaceState.id()}/accounts`
	);
	if (result.status !== 'success') return result;
	return { status: 'success', data: result.data.accounts.map(toAccount) };
}

export type CreateInput = { name: string; parentId: string; currency: string };

export async function create(input: CreateInput): Promise<JSend<Account>> {
	const result = await apiFetch<WireAccount>(
		`/workspaces/${workspaceState.id()}/accounts`,
		{
			method: 'POST',
			body: JSON.stringify({
				name: input.name,
				parent_id: input.parentId,
				currency: input.currency
			})
		}
	);
	if (result.status !== 'success') return result;
	return { status: 'success', data: toAccount(result.data) };
}

export async function rename(id: string, name: string): Promise<JSend<Account>> {
	const result = await apiFetch<WireAccount>(
		`/workspaces/${workspaceState.id()}/accounts/${id}`,
		{
			method: 'PATCH',
			body: JSON.stringify({ name })
		}
	);
	if (result.status !== 'success') return result;
	return { status: 'success', data: toAccount(result.data) };
}

export function remove(id: string): Promise<JSend<null>> {
	return apiFetch<null>(`/workspaces/${workspaceState.id()}/accounts/${id}`, {
		method: 'DELETE'
	});
}
