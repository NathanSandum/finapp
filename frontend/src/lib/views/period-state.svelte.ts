import { shiftPeriod, snapToGranularity, type Granularity } from './period';

type PeriodState = {
	year: number;
	month: number;
	granularity: Granularity;
	initialized: boolean;
};

const state: PeriodState = $state({ year: 0, month: 0, granularity: 'month', initialized: false });

export const periodState = {
	get year() {
		return state.year;
	},
	get month() {
		return state.month;
	},
	get granularity() {
		return state.granularity;
	},
	get initialized() {
		return state.initialized;
	},
	initIfNeeded(year: number, month: number) {
		if (state.initialized) return;
		state.year = year;
		state.month = month;
		state.initialized = true;
	},
	shift(direction: 1 | -1) {
		const next = shiftPeriod(state.year, state.month, state.granularity, direction);
		state.year = next.year;
		state.month = next.month;
	},
	setGranularity(g: Granularity) {
		const snapped = snapToGranularity(state.year, state.month, g);
		state.year = snapped.year;
		state.month = snapped.month;
		state.granularity = g;
	}
};
