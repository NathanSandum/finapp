import { shiftPeriod, snapToGranularity, type Granularity } from './period';

type PeriodState = {
	year: number;
	month: number;
	granularity: Granularity;
};

const state: PeriodState = $state({ year: 0, month: 0, granularity: 'month' });

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
	init(year: number, month: number) {
		state.year = year;
		state.month = month;
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
