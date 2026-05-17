import { cache } from '$lib/api/cache.svelte';
import type { Workspace } from './api';

const STORAGE_KEY = 'finapp.activeWorkspaceId';

const state: { active: Workspace | null; list: Workspace[] } = $state({
	active: null,
	list: []
});

function persist(id: string) {
	try {
		localStorage.setItem(STORAGE_KEY, id);
	} catch {
		// localStorage unavailable; ignore
	}
}

function loadPersisted(): string | null {
	try {
		return localStorage.getItem(STORAGE_KEY);
	} catch {
		return null;
	}
}

export const workspaceState = {
	get active(): Workspace | null {
		return state.active;
	},
	get list(): Workspace[] {
		return state.list;
	},
	id(): string {
		return state.active?.workspace ?? '';
	},
	setList(workspaces: Workspace[]) {
		state.list = workspaces;
		if (state.active) {
			const refreshed = workspaces.find((w) => w.workspace === state.active!.workspace);
			if (refreshed) state.active = refreshed;
		}
	},
	setActive(workspace: Workspace) {
		const switched = state.active?.workspace !== workspace.workspace;
		state.active = workspace;
		persist(workspace.workspace);
		if (switched) cache.invalidateAll();
	},
	pickInitial(workspaces: Workspace[]): Workspace | null {
		if (workspaces.length === 0) return null;
		const persistedId = loadPersisted();
		return workspaces.find((w) => w.workspace === persistedId) ?? workspaces[0];
	},
	clear() {
		state.active = null;
		state.list = [];
	}
};
