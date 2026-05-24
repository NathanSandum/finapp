import { bootstrap, recurring } from '$lib/data';
import { workspaceState } from './state.svelte';
import { listWorkspaces, type Workspace } from './api';
import { latestActivityPeriod } from '$lib/views/period';
import { periodState } from '$lib/views/period-state.svelte';
import type { JSend } from '$lib/tabs/types';

const state = $state({ ready: false });

export const bootState = {
	get ready() {
		return state.ready;
	},
	reset() {
		state.ready = false;
	}
};

export async function bootWorkspace(): Promise<JSend<{ workspace: Workspace }>> {
	const workspacesResult = await listWorkspaces();
	if (workspacesResult.status !== 'success') return workspacesResult;

	workspaceState.setList(workspacesResult.data.workspaces);
	const initial = workspaceState.pickInitial(workspacesResult.data.workspaces);
	if (!initial) {
		return { status: 'fail', data: { message: 'No workspaces available' } };
	}
	workspaceState.setActive(initial);

	// Hybrid materialization: fire any past-due recurring rules into real entries
	// before loading downstream data. Errors are logged but non-fatal — the app
	// still boots if the runs endpoint is unreachable.
	const runResult = await recurring.runScheduled();
	if (runResult.status !== 'success') {
		console.warn('recurring run failed', runResult);
	}

	// One bundle fetch hydrates entries + accounts + anchors + recurring. Tab
	// loaders mounting next will see cached data and not hit the network.
	const bootstrapResult = await bootstrap();
	if (bootstrapResult.status !== 'success') return bootstrapResult;

	const period = latestActivityPeriod(bootstrapResult.data.entries);
	periodState.init(period.year, period.month);
	state.ready = true;

	return { status: 'success', data: { workspace: initial } };
}
