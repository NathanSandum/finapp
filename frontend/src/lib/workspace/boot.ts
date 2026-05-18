import { cache } from '$lib/api/cache.svelte';
import { workspaceState } from './state.svelte';
import { listWorkspaces, type Workspace } from './api';
import { load as loadOverview } from '$lib/views/overview/loader';
import { latestActivityPeriod } from '$lib/views/period';
import { periodState } from '$lib/views/period-state.svelte';
import type { JSend } from '$lib/tabs/types';

export async function bootWorkspace(): Promise<JSend<{ workspace: Workspace }>> {
	const workspacesResult = await listWorkspaces();
	if (workspacesResult.status !== 'success') return workspacesResult;

	workspaceState.setList(workspacesResult.data.workspaces);
	const initial = workspaceState.pickInitial(workspacesResult.data.workspaces);
	if (!initial) {
		return { status: 'fail', data: { message: 'No workspaces available' } };
	}
	workspaceState.setActive(initial);

	const overviewResult = await cache.load('overview', loadOverview);
	if (overviewResult.status !== 'success') return overviewResult;

	const period = latestActivityPeriod(overviewResult.data.entries);
	periodState.init(period.year, period.month);

	return { status: 'success', data: { workspace: initial } };
}
