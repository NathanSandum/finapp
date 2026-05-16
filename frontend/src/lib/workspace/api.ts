import { apiFetch } from '$lib/api/client';
import type { JSend } from '$lib/tabs/types';

export type Workspace = {
	workspace: string;
	name: string;
	created_at: string;
	updated_at: string;
};

export function listWorkspaces(): Promise<JSend<{ workspaces: Workspace[] }>> {
	return apiFetch<{ workspaces: Workspace[] }>('/workspaces');
}

export function createWorkspace(name: string): Promise<JSend<Workspace>> {
	return apiFetch<Workspace>('/workspaces', {
		method: 'POST',
		body: JSON.stringify({ name })
	});
}

export function renameWorkspace(
	id: string,
	name: string
): Promise<JSend<{ workspace: string; name: string }>> {
	return apiFetch<{ workspace: string; name: string }>(`/workspaces/${id}`, {
		method: 'PUT',
		body: JSON.stringify({ name })
	});
}
