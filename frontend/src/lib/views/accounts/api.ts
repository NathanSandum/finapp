import { apiFetch, DEV_WORKSPACE } from '$lib/api/client';
import type { JSend } from '$lib/tabs/types';
import type { Account } from '$lib/views/entries/loader';

export function createAccount(input: {
	name: string;
	parent_id: string;
	currency: string;
}): Promise<JSend<Account>> {
	return apiFetch<Account>(`/workspaces/${DEV_WORKSPACE}/accounts`, {
		method: 'POST',
		body: JSON.stringify(input)
	});
}

export function renameAccount(id: string, name: string): Promise<JSend<Account>> {
	return apiFetch<Account>(`/workspaces/${DEV_WORKSPACE}/accounts/${id}`, {
		method: 'PATCH',
		body: JSON.stringify({ name })
	});
}

export function deleteAccount(id: string): Promise<JSend<null>> {
	return apiFetch<null>(`/workspaces/${DEV_WORKSPACE}/accounts/${id}`, {
		method: 'DELETE'
	});
}
