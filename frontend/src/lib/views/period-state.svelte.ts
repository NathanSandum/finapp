import { shiftPeriod, snapToGranularity, type Granularity } from './period';
import { todayYearMonth } from './today';

type PeriodState = {
	year: number;
	month: number;
	granularity: Granularity;
};

const state: PeriodState = $state({ year: 0, month: 0, granularity: 'month' });

function isAtCurrent(): boolean {
	const t = todayYearMonth();
	const snapNow = snapToGranularity(t.year, t.month, state.granularity);
	const snapState = snapToGranularity(state.year, state.month, state.granularity);
	if (state.granularity === 'year') return snapNow.year === snapState.year;
	return snapNow.year === snapState.year && snapNow.month === snapState.month;
}

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
	get isAtCurrent(): boolean {
		return isAtCurrent();
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
		state.granularity = g;
	},
	goToCurrent() {
		const t = todayYearMonth();
		const snapped = snapToGranularity(t.year, t.month, state.granularity);
		state.year = snapped.year;
		state.month = snapped.month;
	}
};
