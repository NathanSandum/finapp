const state: { currentId: string } = $state({ currentId: 'overview' });

export const tabState = {
	get(): string {
		return state.currentId;
	},
	set(id: string) {
		state.currentId = id;
	}
};
