import { goto } from '$app/navigation';
import { apiFetchMe, apiLogout, apiSignIn, type UserPayload } from '$lib/api/auth';
import { authState } from '$lib/api/state.svelte';
import { clearAll } from '$lib/data';
import { workspaceState } from '$lib/workspace/state.svelte';
import { bootState, bootWorkspace } from '$lib/workspace/boot.svelte';
import { tabState } from '$lib/tabs/state.svelte';
import { overviewEntrance } from '$lib/views/overview/entrance';
import type { JSend } from '$lib/tabs/types';

export async function fetchMe(): Promise<JSend<UserPayload>> {
	const result = await apiFetchMe();
	if (result.status === 'success') authState.set(result.data.user);
	return result;
}

export async function signIn(email: string, password: string): Promise<JSend<unknown>> {
	const result = await apiSignIn(email, password);
	if (result.status !== 'success') return result;
	clearAll();
	authState.set(result.data.user);
	const bootResult = await bootWorkspace();
	if (bootResult.status !== 'success') return bootResult;
	tabState.set('overview');
	overviewEntrance.arm();
	return result;
}

export async function logout(): Promise<void> {
	await apiLogout();
	// Navigate first so the workspace shell unmounts before any state-clear
	// triggers reactive cascades — otherwise mounted TabPanels see an empty
	// cache, refire their loaders with workspace="", and hit /workspaces//...
	await goto('/signin');
	authState.clear();
	workspaceState.clear();
	bootState.reset();
	clearAll();
}
