export type SchedulePresetId = 'weekly' | 'biweekly' | 'monthly-first' | 'monthly-last' | 'yearly';

export type SchedulePreset = {
	id: SchedulePresetId;
	label: string;
	rrule: string;
};

export const SCHEDULE_PRESETS: SchedulePreset[] = [
	{ id: 'weekly', label: 'Weekly', rrule: 'FREQ=WEEKLY' },
	{ id: 'biweekly', label: 'Every 2 weeks', rrule: 'FREQ=WEEKLY;INTERVAL=2' },
	{ id: 'monthly-first', label: 'Monthly (1st)', rrule: 'FREQ=MONTHLY;BYMONTHDAY=1' },
	{ id: 'monthly-last', label: 'Monthly (last day)', rrule: 'FREQ=MONTHLY;BYMONTHDAY=-1' },
	{ id: 'yearly', label: 'Yearly', rrule: 'FREQ=YEARLY' }
];

const PRESET_BY_RRULE: Record<string, SchedulePreset> = Object.fromEntries(
	SCHEDULE_PRESETS.map((p) => [p.rrule, p])
);

export function presetByRrule(rrule: string): SchedulePreset | undefined {
	return PRESET_BY_RRULE[rrule];
}

export function scheduleLabel(rrule: string): string {
	return PRESET_BY_RRULE[rrule]?.label ?? rrule;
}
