import { afterEach, describe, expect, it, vi } from 'vitest';
import { today, todayLabel, todayYearMonth } from './today';

describe('today', () => {
	afterEach(() => {
		vi.useRealTimers();
	});

	it('returns local-time YYYY-MM-DD', () => {
		vi.useFakeTimers();
		vi.setSystemTime(new Date(2026, 4, 12, 10, 30));
		expect(today()).toBe('2026-05-12');
	});

	it('pads single-digit month and day', () => {
		vi.useFakeTimers();
		vi.setSystemTime(new Date(2026, 0, 3, 12, 0));
		expect(today()).toBe('2026-01-03');
	});
});

describe('todayLabel', () => {
	afterEach(() => {
		vi.useRealTimers();
	});

	it('formats as "Weekday · Mon D, YYYY"', () => {
		vi.useFakeTimers();
		vi.setSystemTime(new Date(2026, 4, 12, 10, 30));
		expect(todayLabel()).toBe('Tue · May 12, 2026');
	});
});

describe('todayYearMonth', () => {
	afterEach(() => {
		vi.useRealTimers();
	});

	it('returns local-time year and 1-indexed month', () => {
		vi.useFakeTimers();
		vi.setSystemTime(new Date(2026, 4, 12, 10, 30));
		expect(todayYearMonth()).toEqual({ year: 2026, month: 5 });
	});
});
