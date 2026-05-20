<script lang="ts">
	import { SCHEDULE_PRESETS, type SchedulePresetId } from './schedules';
	import { INPUT, MONO, LABEL } from './field-classes';

	let {
		presetId = $bindable(),
		dtstart = $bindable()
	}: {
		presetId: SchedulePresetId;
		dtstart: string;
	} = $props();
</script>

<div class="grid grid-cols-[1fr_auto] gap-3">
	<label class="flex flex-col gap-1">
		<span class={LABEL}>Schedule</span>
		<select bind:value={presetId} class={INPUT} required>
			{#each SCHEDULE_PRESETS as preset (preset.id)}
				<option value={preset.id}>{preset.label}</option>
			{/each}
		</select>
	</label>
	<label class="flex flex-col gap-1">
		<span class={LABEL}>Starts</span>
		<input type="date" bind:value={dtstart} class={MONO} required />
	</label>
</div>
