import { apiFetch } from './client';
import { workspaceState } from '$lib/workspace/state.svelte';
import type { JSend } from '$lib/tabs/types';
import type { Anchors } from '$lib/data/types';

// Wire shape matches Anchors today; if it diverges, translate here.
export function fetchAll(): Promise<JSend<Anchors>> {
	return apiFetch<Anchors>(`/workspaces/${workspaceState.id()}/anchors`);
}
