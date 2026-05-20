<script lang="ts">
	import { INPUT, LABEL } from './field-classes';

	let {
		label,
		value = $bindable(),
		placeholder = ''
	}: {
		label: string;
		value: string;
		placeholder?: string;
	} = $props();
</script>

<label class="flex flex-col gap-1">
	<span class={LABEL}>{label}</span>
	<input type="text" bind:value {placeholder} class={INPUT} />
</label>
