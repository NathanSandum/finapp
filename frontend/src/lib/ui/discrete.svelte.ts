const STORAGE_KEY = 'discreteMode';

function loadInitial(): boolean {
	if (typeof localStorage === 'undefined') return false;
	return localStorage.getItem(STORAGE_KEY) === '1';
}

const state = $state({ on: loadInitial() });

export function getDiscrete(): boolean {
	return state.on;
}

export function setDiscrete(value: boolean) {
	state.on = value;
	if (typeof localStorage === 'undefined') return;
	if (value) localStorage.setItem(STORAGE_KEY, '1');
	else localStorage.removeItem(STORAGE_KEY);
}
