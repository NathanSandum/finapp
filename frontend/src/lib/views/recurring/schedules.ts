// Wire-format schedule tags. The backend owns the allowlist and the mapping
// from id → RRULE; the frontend only handles ids + display labels. Do not
// surface the RRULE strings outside the backend.

export type SchedulePresetId =
	| 'weekly'
	| 'biweekly'
	| 'monthly-first'
	| 'monthly-last'
	| 'yearly';

export type SchedulePreset = {
	id: SchedulePresetId;
	label: string;
};

export const SCHEDULE_PRESETS: SchedulePreset[] = [
	{ id: 'weekly', label: 'Weekly' },
	{ id: 'biweekly', label: 'Every 2 weeks' },
	{ id: 'monthly-first', label: 'Monthly (1st)' },
	{ id: 'monthly-last', label: 'Monthly (last day)' },
	{ id: 'yearly', label: 'Yearly' }
];

const LABEL_BY_ID: Record<SchedulePresetId, string> = Object.fromEntries(
	SCHEDULE_PRESETS.map((p) => [p.id, p.label])
) as Record<SchedulePresetId, string>;

export function scheduleLabel(id: SchedulePresetId | null): string {
	if (id === null) return 'Custom';
	return LABEL_BY_ID[id] ?? id;
}
