const STORAGE_KEY = 'discreteMode';

function initial(): boolean {
	if (typeof localStorage === 'undefined') return false;
	return localStorage.getItem(STORAGE_KEY) === '1';
}

const state = $state({ on: initial() });

export const discreteState = {
	get on() {
		return state.on;
	},
	set(value: boolean) {
		state.on = value;
		if (typeof localStorage !== 'undefined') {
			if (value) localStorage.setItem(STORAGE_KEY, '1');
			else localStorage.removeItem(STORAGE_KEY);
		}
	},
	toggle() {
		this.set(!state.on);
	}
};
